use rusqlite::Connection;
use std::path::PathBuf;
use crate::modules::compute::year_over_year::Candle;

/// Seconds in a day.
const SECS_PER_DAY: i64 = 86_400;

/// Return the UTC-midnight timestamp (start of day) that contains `ts`.
fn start_of_day(ts: i64) -> i64 {
    ts - (ts % SECS_PER_DAY)
}

/// Resolve the inclusive candle-day window `[start, end]` for a halving
/// period. `now` is injected for testability and represents the current time
/// on the boundary between completed and live halvings.
///
/// - The period starts on the day of this halving's block.
/// - The period ends on the day of the next halving's block (inclusive), so
///   that boundary candle is this period's close and the next period's open.
/// - If the next halving has no timestamp, this is the live halving, so the
///   period runs to today ("halving-to-date").
/// - If this halving has no timestamp, it has not happened yet, so `None`.
fn range_from_conn(conn: &Connection, halving_number: u32, now: i64) -> Option<(i64, i64)> {
    let start_ts: Option<i64> = conn
        .query_row(
            "SELECT timestamp FROM halve_blocks WHERE halving_number = ?1",
            [i64::from(halving_number)],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    let start_ts = start_ts?;

    let end_ts: Option<i64> = conn
        .query_row(
            "SELECT timestamp FROM halve_blocks WHERE halving_number = ?1",
            [i64::from(halving_number + 1)],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let end = match end_ts {
        Some(ts) => start_of_day(ts),
        None => start_of_day(now),
    };

    Some((start_of_day(start_ts), end))
}

/// Query the daily candles that fall inside the inclusive day window.
fn query_candles(conn: &Connection, start: i64, end: i64) -> Vec<Candle> {
    let mut stmt = match conn.prepare(
        "SELECT timestamp, open, high, low, close, volume
         FROM daily_candles
         WHERE timestamp >= ?1 AND timestamp <= ?2
         ORDER BY timestamp ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([start, end], |row| {
        Ok(Candle {
            timestamp: row.get(0)?,
            open: row.get(1)?,
            high: row.get(2)?,
            low: row.get(3)?,
            close: row.get(4)?,
            volume: row.get(5)?,
        })
    });

    match rows {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Combine the halving block DB and the candle DB into a period's candles.
fn query_period(
    blocks: &Connection,
    candles: &Connection,
    halving_number: u32,
    now: i64,
) -> Vec<Candle> {
    match range_from_conn(blocks, halving_number, now) {
        Some((start, end)) => query_candles(candles, start, end),
        None => Vec::new(),
    }
}

/// Path to the mempool block database (`halve_blocks` table).
fn blocks_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Mempool").join("blocks.db")
}

/// Path to the daily candle database (`daily_candles` table).
fn candles_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Exchange").join("btcusd.db")
}

/// Current unix time in seconds.
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Public: resolve the inclusive candle-day window for a halving period.
///
/// Returns `None` when the halving has not occurred yet.
pub fn halving_period_range(halving_number: u32) -> Option<(i64, i64)> {
    let conn = Connection::open(blocks_db_path()).ok()?;
    range_from_conn(&conn, halving_number, now_ts())
}

/// Public: the block subsidy in satoshis for a specific halving.
///
/// Returns `None` when the database or the row is unavailable. Each halving
/// row is seeded with its subsidy during the mempool sync.
pub fn halving_subsidy_sat(halving_number: u32) -> Option<i64> {
    let conn = Connection::open(blocks_db_path()).ok()?;
    conn.query_row(
        "SELECT subsidy FROM halve_blocks WHERE halving_number = ?1",
        [i64::from(halving_number)],
        |row| row.get(0),
    )
    .ok()
    .flatten()
}

/// Public: the block subsidy for a specific halving, formatted in BTC.
///
/// Returns an em-dash when the database or the row is unavailable or the
/// subsidy is zero.
pub fn halving_subsidy_btc(halving_number: u32) -> String {
    match halving_subsidy_sat(halving_number) {
        Some(sat) if sat > 0 => format!("{:.8}", sat as f64 / 100_000_000.0),
        _ => "\u{2014}".to_string(),
    }
}

/// Public: daily candles for a halving period, ascending by date.
///
/// Returns an empty set when the halving is in the future or a database is
/// unavailable. Empty candles naturally produce dash metrics in the UI.
pub fn halving_period_candles(halving_number: u32) -> Vec<Candle> {
    let blocks = match Connection::open(blocks_db_path()) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let candles = match Connection::open(candles_db_path()) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    query_period(&blocks, &candles, halving_number, now_ts())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_blocks(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE halve_blocks (
                halving_number INTEGER PRIMARY KEY,
                height        INTEGER NOT NULL,
                timestamp     INTEGER,
                subsidy       INTEGER NOT NULL
            );",
        )
        .unwrap();
        // H-1, H-2, H-3 are reached. H-4 and H-5 rows exist with NULL
        // timestamps (not yet mined), matching how the mempool sync seeds them.
        let reached = [
            (1, 1_000_000_000i64, 5_000_000_000i64 / 2),
            (2, 1_500_000_000i64, 5_000_000_000i64 / 4),
            (3, 2_000_000_000i64, 5_000_000_000i64 / 8),
        ];
        for (n, ts, subsidy) in reached {
            conn.execute(
                "INSERT INTO halve_blocks (halving_number, height, timestamp, subsidy)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![n, n * 210_000, ts, subsidy],
            )
            .unwrap();
        }
        for n in 4..=5 {
            conn.execute(
                "INSERT INTO halve_blocks (halving_number, height, timestamp, subsidy)
                 VALUES (?1, ?2, NULL, ?3)",
                rusqlite::params![n, n * 210_000, 5_000_000_000i64 / (1i64 << n)],
            )
            .unwrap();
        }
    }

    fn seed_candles(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE daily_candles (
                timestamp INTEGER PRIMARY KEY,
                open      REAL NOT NULL,
                high      REAL NOT NULL,
                low       REAL NOT NULL,
                close     REAL NOT NULL,
                volume    REAL NOT NULL
            );",
        )
        .unwrap();

        let h1_day = start_of_day(1_000_000_000);
        let h2_day = start_of_day(1_500_000_000);
        let h3_day = start_of_day(2_000_000_000);

        let rows = [
            (h1_day, 10.0, 12.0, 9.0, 11.0, 100.0),
            (h1_day + SECS_PER_DAY, 11.0, 13.0, 10.0, 12.0, 150.0),
            // Boundary candle: H-1 close AND H-2 open.
            (h2_day, 12.0, 14.0, 11.0, 13.0, 200.0),
            (h2_day + SECS_PER_DAY, 13.0, 15.0, 12.0, 14.0, 250.0),
            (h3_day, 14.0, 16.0, 13.0, 15.0, 300.0),
        ];
        for c in rows {
            conn.execute(
                "INSERT INTO daily_candles (timestamp, open, high, low, close, volume)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![c.0, c.1, c.2, c.3, c.4, c.5],
            )
            .unwrap();
        }
    }

    #[test]
    fn past_halving_end_inclusive() {
        let blocks = Connection::open_in_memory().unwrap();
        seed_blocks(&blocks);
        let candles = Connection::open_in_memory().unwrap();
        seed_candles(&candles);

        // H-1 uses H-2's block day inclusive. The boundary candle is included.
        let result = query_period(&blocks, &candles, 1, 99_999_999_999);
        let days: Vec<i64> = result.iter().map(|c| c.timestamp).collect();
        let h1_day = start_of_day(1_000_000_000);
        let h2_day = start_of_day(1_500_000_000);
        assert_eq!(
            days,
            vec![h1_day, h1_day + SECS_PER_DAY, h2_day]
        );
    }

    #[test]
    fn next_halving_starts_at_boundary() {
        let blocks = Connection::open_in_memory().unwrap();
        seed_blocks(&blocks);
        let candles = Connection::open_in_memory().unwrap();
        seed_candles(&candles);

        // H-2 starts on the same boundary candle that closed H-1, and is
        // itself end-inclusive on H-3's boundary day.
        let result = query_period(&blocks, &candles, 2, 99_999_999_999);
        let days: Vec<i64> = result.iter().map(|c| c.timestamp).collect();
        let h2_day = start_of_day(1_500_000_000);
        let h3_day = start_of_day(2_000_000_000);
        assert_eq!(
            days,
            vec![h2_day, h2_day + SECS_PER_DAY, h3_day]
        );
    }

    #[test]
    fn future_halving_returns_empty() {
        let blocks = Connection::open_in_memory().unwrap();
        seed_blocks(&blocks);
        let candles = Connection::open_in_memory().unwrap();
        seed_candles(&candles);

        // H-5 has a row but a NULL timestamp (not mined yet).
        let result = query_period(&blocks, &candles, 5, 99_999_999_999);
        assert!(result.is_empty());
    }

    #[test]
    fn live_halving_extends_to_today() {
        let blocks = Connection::open_in_memory().unwrap();
        seed_blocks(&blocks);
        let candles = Connection::open_in_memory().unwrap();
        seed_candles(&candles);

        // H-3 is reached (H-4 not yet), so it is the live halving.
        let h3_day = start_of_day(2_000_000_000);
        let now = h3_day + 5 * SECS_PER_DAY;
        let (start, end) = range_from_conn(&blocks, 3, now).unwrap();
        assert_eq!(start, h3_day);
        assert_eq!(end, start_of_day(now));

        let result = query_period(&blocks, &candles, 3, now);
        assert!(!result.is_empty());
        for c in &result {
            assert!(c.timestamp >= start && c.timestamp <= end);
        }
    }
}