use crate::modules::app_data_dir::btcusd_db_path;
use rusqlite::Connection;

/// A single daily OHLC candle.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Query the trailing 365 days of daily candles from the BTC/USD database.
///
/// Returns candles ordered by timestamp ascending. The range is computed
/// as: `[latest_timestamp - 364 days, latest_timestamp]`.
pub fn trailing_365_candles() -> Vec<Candle> {
    let conn = match Connection::open(btcusd_db_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[year_over_year] failed to open database: {e}");
            return Vec::new();
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT timestamp, open, high, low, close, volume
         FROM daily_candles
         WHERE timestamp >= (SELECT MAX(timestamp) FROM daily_candles) - 365 * 86400
         ORDER BY timestamp ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[year_over_year] failed to prepare query: {e}");
            return Vec::new();
        }
    };

    let candles: Vec<Candle> = match stmt.query_map([], |row| {
        Ok(Candle {
            timestamp: row.get(0)?,
            open: row.get(1)?,
            high: row.get(2)?,
            low: row.get(3)?,
            close: row.get(4)?,
            volume: row.get(5)?,
        })
    }) {
        Ok(rows) => rows.filter_map(std::result::Result::ok).collect(),
        Err(e) => {
            eprintln!("[year_over_year] failed to query candles: {e}");
            return Vec::new();
        }
    };

    candles
}
