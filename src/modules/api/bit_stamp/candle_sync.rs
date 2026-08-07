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
/// - On first run, backfills from Nov 1 2012 to today (inclusive).
/// - On subsequent runs, fetches only the gap (if any) since the latest candle.
/// - Skips entirely if the latest candle is today.
/// - Today's candle is overwritten on each sync so partial-day volume updates.
pub fn fetch_and_store() {
    let db_path = db_path();

    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[bitstamp] failed to open database {db_path:?}: {e}");
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
        );
        CREATE TABLE IF NOT EXISTS metadata (
            key   TEXT PRIMARY KEY,
            value INTEGER NOT NULL
        );",
    ) {
        eprintln!("[bitstamp] failed to create tables: {e}");
        return;
    }

    // Refresh today's volume if the 1-hour cooldown has expired.
    // This runs before the gap/early-return logic so it works even when
    // the DB already has today's candle (e.g. on app restart).
    refresh_today_volume_if_stale(&conn);

    // Determine the latest candle we already have.
    let latest_ts: Option<i64> = conn
        .query_row("SELECT MAX(timestamp) FROM daily_candles", [], |row| {
            row.get(0)
        })
        .ok()
        .flatten();

    let start_ts = latest_ts.map_or(EPOCH_START, |t| t + 86_400);

    // Today's midnight (00:00:00 UTC) — includes the current incomplete candle.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let today_midnight = now - (now % 86_400);

    if start_ts > today_midnight {
        eprintln!(
            "[bitstamp] already up to date (latest: {})",
            latest_ts.unwrap_or(0)
        );
        return;
    }

    // Compute how many days we need to fetch, with a 1-candle buffer.
    // We request up to today_midnight inclusive.
    let gap_days = (today_midnight - start_ts) / 86_400 + 1;

    if gap_days <= 0 {
        eprintln!(
            "[bitstamp] already up to date (latest: {})",
            latest_ts.unwrap_or(0)
        );
        return;
    }

    eprintln!(
        "[bitstamp] gap of {gap_days} day(s) to fill (with 1-candle buffer)"
    );

    // Paginate backwards from today_midnight, using an appropriate limit per page.
    let mut total_inserted = 0u64;
    let mut remaining = gap_days + 1; // include buffer
    let mut cursor = today_midnight;

    while remaining > 0 {
        let limit = remaining.min(PAGE_SIZE);
        let batch_start = cursor - (limit - 1) * 86_400;

        eprintln!(
            "[bitstamp] fetching {limit} candles starting at {batch_start}"
        );

        let Some(candles) = fetch_page(batch_start, limit) else {
            eprintln!("[bitstamp] API error at start={batch_start}, aborting");
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
        let earliest_in_batch = candles.first().map_or(batch_start, |c| c.timestamp);

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
        "[bitstamp] sync complete – {total_inserted} new candles stored"
    );
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Return the path to the `SQLite` database.
fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Exchange").join("btcusd.db")
}

/// Fetch up to `limit` daily candles starting at `start_ts` (unix seconds).
/// `limit` must not exceed 1000 (Bitstamp API limit).
/// Returns `None` on any network or parse error.
fn fetch_page(start_ts: i64, limit: i64) -> Option<Vec<Candle>> {
    let url = format!(
        "https://www.bitstamp.net/api/v2/ohlc/btcusd/?step=86400&limit={limit}&start={start_ts}"
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

/// Check the metadata cooldown and fetch today's candle if expired.
fn refresh_today_volume_if_stale(conn: &Connection) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let last_fetch: Option<i64> = conn
        .query_row(
            "SELECT value FROM metadata WHERE key = 'last_volume_fetch'",
            [],
            |row| row.get(0),
        )
        .ok();

    let should_refetch = match last_fetch {
        None => true,
        Some(ts) => now - ts >= 3600,
    };

    if !should_refetch {
        return;
    }

    let today_midnight = now - (now % 86_400);
    let Some(candles) = fetch_page(today_midnight, 1) else {
        eprintln!("[bitstamp] failed to fetch today's candle for volume refresh");
        return;
    };
    store_candles(conn, &candles);

    if let Err(e) = conn.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('last_volume_fetch', ?1)",
        rusqlite::params![now],
    ) {
        eprintln!("[bitstamp] failed to update volume fetch timestamp: {e}");
    }
}

/// Fetch only today's partial candle from Bitstamp and overwrite it in the DB.
///
/// This is called periodically (every hour) so the volume for the current
/// incomplete day stays reasonably up-to-date without re-fetching the entire
/// 365-day range.
pub fn update_today_volume() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let today_midnight = now - (now % 86_400);

    let Some(candles) = fetch_page(today_midnight, 1) else {
        eprintln!("[bitstamp] failed to fetch today's candle for volume update");
        return;
    };

    let conn = match Connection::open(db_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[bitstamp] failed to open database for volume update: {e}"
            );
            return;
        }
    };

    store_candles(&conn, &candles);
}

/// Insert candles into the database, skipping existing timestamps.
/// Returns the number of rows actually inserted.
fn store_candles(conn: &Connection, candles: &[Candle]) -> u64 {
    let mut count = 0u64;

    // Use a transaction for performance when inserting many rows.
    if let Err(e) = conn.execute_batch("BEGIN TRANSACTION") {
        eprintln!("[bitstamp] failed to begin transaction: {e}");
        return 0;
    }

    for c in candles {
        match conn.execute(
            "INSERT OR REPLACE INTO daily_candles (timestamp, open, high, low, close, volume)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![c.timestamp, c.open, c.high, c.low, c.close, c.volume],
        ) {
            Ok(rows) => count += rows as u64,
            Err(e) => eprintln!("[bitstamp] failed to insert candle {}: {}", c.timestamp, e),
        }
    }

    if let Err(e) = conn.execute_batch("COMMIT") {
        eprintln!("[bitstamp] failed to commit transaction: {e}");
    }

    count
}
