use crate::modules::app_data_dir::btcusd_db_path;
use rusqlite::Connection;
use std::sync::OnceLock;

/// Insert thousands commas into an unsigned integer's decimal string.
fn group_thousands(whole: u64) -> String {
    let s = whole.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Format a number with thousands commas and 2 fixed decimal places.
fn fmt_usd(value: f64) -> String {
    let whole = value.trunc() as u64;
    let cents = ((value - value.trunc()) * 100.0).round() as u64;

    let mut grouped = group_thousands(whole);
    // `.` plus two cents digits.
    grouped.reserve(3);
    grouped.push('.');
    grouped.push_str(&format!("{cents:02}"));
    grouped
}

/// Format a number with thousands commas (no decimal places).
fn fmt_whole(value: f64) -> String {
    group_thousands(value.round() as u64)
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
        format!("${value:.8}")
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
        let conn = Connection::open(btcusd_db_path()).ok()?;
        conn.query_row("SELECT MAX(high) FROM daily_candles", [], |row| row.get(0))
            .ok()
            .flatten()
    })
}

/// The historical all-time high from the database, queried once and cached
/// for the session.
pub fn db_high() -> Option<f64> {
    db_all_time_high()
}

/// Format an asset high as USD, returning `"—"` when `value` is non-positive
/// (no data yet).
pub fn fmt_high(value: f64) -> String {
    if value <= 0.0 {
        return "\u{2014}".to_string();
    }
    format!("${}", fmt_usd(value))
}
