//! Compute the Brass Tacks comparison between Bitcoin and the S&P 500.
//!
//! All start values are fixed constants, so this module never reads the
//! database. Only the current prices are live.

/// The S&P 500 close on the comparison start date (August 18, 2011).
pub const SP500_START_VALUE: f64 = 1140.65;

/// The BTC close on the comparison start date (August 18, 2011).
pub const BTC_START_VALUE: f64 = 10.90;

/// The comparison start date, shown in the dialog.
pub const START_DATE_LABEL: &str = "August 18, 2011";

/// Percentage change from `start` to `current`.
///
/// Returns `None` when either value is not finite, or when `start` is zero.
/// The caller then shows an em-dash instead of a bad or infinite change.
pub fn pct_change(start: f64, current: f64) -> Option<f64> {
    if !start.is_finite() || !current.is_finite() || start == 0.0 {
        return None;
    }
    Some((current - start) / start * 100.0)
}

/// How far BTC outperforms the S&P 500, in percentage points.
///
/// A positive value means BTC is ahead. A negative value means BTC is behind.
/// Returns `None` when either change is unavailable.
pub fn outperformance(btc_pct: Option<f64>, sp500_pct: Option<f64>) -> Option<f64> {
    Some(btc_pct? - sp500_pct?)
}
