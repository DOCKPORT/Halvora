use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::OnceLock;

/// Format a number with thousands commas and 2 fixed decimal places.
fn fmt_usd(value: f64) -> String {
    let whole = value.trunc() as u64;
    let cents = ((value - value.trunc()) * 100.0).round() as u64;

    let whole_str = whole.to_string();
    let mut result = String::with_capacity(whole_str.len() + whole_str.len() / 3 + 3);
    for (i, c) in whole_str.chars().enumerate() {
        if i > 0 && (whole_str.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.push('.');
    result.push_str(&format!("{:02}", cents));
    result
}

/// Format a number with thousands commas (no decimal places).
fn fmt_whole(value: f64) -> String {
    let whole = value.round() as u64;
    let s = whole.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Compute the current USD value of the block subsidy.
///
/// `subsidy_value = live_price * (subsidy_sat / 100_000_000.0)`
///
/// Returns a formatted string like `"$332,093.10"` or `"—"` if price is unavailable.
pub fn subsidy_value(live_price: Option<f64>, subsidy_sat: i64) -> String {
    let price = match live_price {
        Some(p) if p > 0.0 => p,
        _ => return "\u{2014}".to_string(),
    };

    let subsidy_btc = subsidy_sat as f64 / 100_000_000.0;
    let value = price * subsidy_btc;

    if value >= 1.0 {
        format!("${}", fmt_usd(value))
    } else {
        format!("${:.8}", value)
    }
}

/// Compute how many satoshis one USD buys.
///
/// `sats_per_usd = 100_000_000 / live_price`
///
/// Returns a formatted whole-number string like `"1,505"` or `"—"` if price is unavailable.
pub fn sats_per_usd(live_price: Option<f64>) -> String {
    let price = match live_price {
        Some(p) if p > 0.0 => p,
        _ => return "\u{2014}".to_string(),
    };

    let sats = 100_000_000.0 / price;
    fmt_whole(sats)
}

/// Return the DB all-time high, cached after the first query so we only
/// hit the filesystem once per process lifetime.
fn db_all_time_high() -> Option<f64> {
    static CACHED: OnceLock<Option<f64>> = OnceLock::new();
    *CACHED.get_or_init(|| {
        let db_path = db_path();
        let conn = Connection::open(&db_path).ok()?;
        conn.query_row(
            "SELECT MAX(high) FROM daily_candles",
            [],
            |row| row.get(0),
        )
        .ok()
        .flatten()
    })
}

/// Return the effective all-time high — the greater of the historical DB max
/// and the current live WebSocket price (if provided).
///
/// When `live_price` exceeds the DB record, it overrides the displayed value.
/// The DB record is queried once and cached for the session.
///
/// Returns a formatted string like `"$73,750.07"` or `"—"` if no data.
pub fn all_time_high(live_price: Option<f64>) -> String {
    let db_max = db_all_time_high();

    let effective = match (db_max, live_price) {
        (Some(db), Some(live)) => db.max(live),
        (Some(db), None) => db,
        (None, Some(live)) => live,
        (None, None) => return "\u{2014}".to_string(),
    };

    format!("${}", fmt_usd(effective))
}

/// Return the path to the BTC/USD OHLC database.
fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Exchange").join("btcusd.db")
}