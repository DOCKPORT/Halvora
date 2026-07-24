use crate::modules::compute::year_over_year::Candle;

/// Compute the visible X range (earliest to latest timestamp).
/// Matches line_chart::x_bounds() for alignment.
pub fn x_bounds(candles: &[Candle]) -> (f64, f64) {
    if candles.is_empty() {
        return (0.0, 1.0);
    }
    let min = candles.first().unwrap().timestamp as f64;
    let max = candles.last().unwrap().timestamp as f64;
    if (min - max).abs() < f64::EPSILON {
        (min - 86_400.0, max + 86_400.0)
    } else {
        (min, max)
    }
}

/// Compute the Y range for volume: [0, max_volume * 1.15].
pub fn y_bounds(candles: &[Candle]) -> (f64, f64) {
    if candles.is_empty() {
        return (0.0, 1.0);
    }
    let max_vol = candles.iter().map(|c| c.volume).fold(0.0_f64, f64::max);
    if max_vol <= 0.0 {
        (0.0, 1.0)
    } else {
        (0.0, max_vol * 1.15)
    }
}
