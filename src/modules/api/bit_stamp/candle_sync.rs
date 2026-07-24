use rusqlite::Connection;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

/// Unix timestamp for Nov 1 2012 00:00:00 UTC — earliest BTC/USD data on Bitstamp.
const EPOCH_START: i64 = 1_351_728_000;

/// Number of daily candles per API request (Bitstamp max is 1000).
const PAGE_SIZE: i64 = 1000;

/// Milliseconds between API requests to avoid rate-limiting.
const REQUEST_DELAY_MS: u64 = 1000;

// ── JSON response shapes ────────────────────────────────────────────────

/// Top-level response from Bitstamp OHLC endpoint.
#[derive(Deserialize)]
struct OhlcResponse {
    data: OhlcData,
}

#[derive(Deserialize)]
struct OhlcData {
    ohlc: Vec<CandleJson>,
}

/// Shape of a single candle returned by the API.
/// All numeric fields come as strings (to preserve precision).
#[derive(Deserialize)]
struct CandleJson {
    timestamp: String,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

/// Parsed candle ready for DB insertion.
struct Candle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

// ── Public entry point ──────────────────────────────────────────────────

/// Fetch BTC/USD daily OHLC candles from Bitstamp and store them in
/// `~/.local/share/Halvora/Exchange/btcusd.db`.
///
/// - On first run, backfills from Nov 1 2012 to yesterday.
/// - On subsequent runs, fetches only the gap (if any) since the latest candle.
/// - Skips entirely if the latest candle is yesterday.
pub fn fetch_and_store() {
    let db_path = db_path();

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[bitstamp] failed to open database {:?}: {}",
                db_path, e
            );
            return;
        }
    };

    if let Err(e) = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS daily_candles (
            timestamp INTEGER PRIMARY KEY,
            open      REAL NOT NULL,
            high      REAL NOT NULL,
            low       REAL NOT NULL,
            close     REAL NOT NULL,
            volume    REAL NOT NULL
        );",
    ) {
        eprintln!("[bitstamp] failed to create table: {}", e);
        return;
    }

    // Determine the latest candle we already have.
    let latest_ts: Option<i64> = conn
        .query_row(
            "SELECT MAX(timestamp) FROM daily_candles",
            [],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let start_ts = latest_ts.map(|t| t + 86_400).unwrap_or(EPOCH_START);

    // Yesterday's midnight (00:00:00 UTC) — the last completed daily candle.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let today_midnight = now - (now % 86_400);
    let yesterday_midnight = today_midnight - 86_400;

    if start_ts > yesterday_midnight {
        eprintln!("[bitstamp] already up to date (latest: {})", latest_ts.unwrap_or(0));
        return;
    }

    // Compute how many days we need to fetch, with a 2-candle buffer.
    // We only request completed candles (up to yesterday_midnight inclusive).
    let gap_days = (yesterday_midnight - start_ts) / 86_400 + 1;

    if gap_days <= 0 {
        eprintln!("[bitstamp] already up to date (latest: {})", latest_ts.unwrap_or(0));
        return;
    }

    eprintln!(
        "[bitstamp] gap of {} day(s) to fill (with 2-candle buffer)",
        gap_days
    );

    // Paginate backwards from yesterday_midnight, using an appropriate limit per page.
    let mut total_inserted = 0u64;
    let mut remaining = gap_days + 2; // include buffer
    let mut cursor = yesterday_midnight;

    while remaining > 0 {
        let limit = remaining.min(PAGE_SIZE);
        let batch_start = cursor - (limit - 1) * 86_400;

        eprintln!(
            "[bitstamp] fetching {} candles starting at {}",
            limit, batch_start
        );

        let Some(candles) = fetch_page(batch_start, limit) else {
            eprintln!(
                "[bitstamp] API error at start={}, aborting",
                batch_start
            );
            break;
        };

        let inserted = store_candles(&conn, &candles);
        total_inserted += inserted;

        eprintln!(
            "[bitstamp] stored {} candles ({} inserted, {} skipped)",
            candles.len(),
            inserted,
            candles.len() - inserted as usize,
        );

        // If this batch already covered down to start_ts, we're done.
        let earliest_in_batch = candles
            .first()
            .map(|c| c.timestamp)
            .unwrap_or(batch_start);

        if earliest_in_batch <= start_ts {
            break;
        }

        // Move cursor back: next batch ends one day before this batch's earliest candle.
        cursor = earliest_in_batch - 86_400;

        // Decrement remaining by how many days this batch actually covered.
        let covered = ((cursor - batch_start) / 86_400) + 1;
        remaining = remaining.saturating_sub(covered);

        // Polite delay between requests.
        std::thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));
    }

    eprintln!(
        "[bitstamp] sync complete – {} new candles stored",
        total_inserted
    );
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Return the path to the SQLite database.
fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Exchange").join("btcusd.db")
}

/// Fetch up to `limit` daily candles starting at `start_ts` (unix seconds).
/// `limit` must not exceed 1000 (Bitstamp API limit).
/// Returns `None` on any network or parse error.
fn fetch_page(start_ts: i64, limit: i64) -> Option<Vec<Candle>> {
    let url = format!(
        "https://www.bitstamp.net/api/v2/ohlc/btcusd/?step=86400&limit={}&start={}",
        limit, start_ts
    );

    let text = reqwest::blocking::get(&url).ok()?.text().ok()?;
    let response: OhlcResponse = serde_json::from_str(&text).ok()?;

    let candles: Vec<Candle> = response
        .data
        .ohlc
        .iter()
        .filter_map(|c| {
            let timestamp = c.timestamp.parse::<i64>().ok()?;
            let open = c.open.parse::<f64>().ok()?;
            let high = c.high.parse::<f64>().ok()?;
            let low = c.low.parse::<f64>().ok()?;
            let close = c.close.parse::<f64>().ok()?;
            let volume = c.volume.parse::<f64>().ok()?;
            Some(Candle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            })
        })
        .collect();

    Some(candles)
}

/// Insert candles into the database, skipping existing timestamps.
/// Returns the number of rows actually inserted.
fn store_candles(conn: &Connection, candles: &[Candle]) -> u64 {
    let mut count = 0u64;

    // Use a transaction for performance when inserting many rows.
    if let Err(e) = conn.execute_batch("BEGIN TRANSACTION") {
        eprintln!("[bitstamp] failed to begin transaction: {}", e);
        return 0;
    }

    for c in candles {
        match conn.execute(
            "INSERT OR IGNORE INTO daily_candles (timestamp, open, high, low, close, volume)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![c.timestamp, c.open, c.high, c.low, c.close, c.volume],
        ) {
            Ok(rows) => count += rows as u64,
            Err(e) => eprintln!(
                "[bitstamp] failed to insert candle {}: {}",
                c.timestamp, e
            ),
        }
    }

    if let Err(e) = conn.execute_batch("COMMIT") {
        eprintln!("[bitstamp] failed to commit transaction: {}", e);
    }

    count
}