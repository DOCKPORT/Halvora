use crate::modules::compute::year_over_year::Candle;

/// Display-ready values for the top metric cards.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub p_l: String,
    pub high: String,
    pub low: String,
    pub draw_down: String,
    pub run_up: String,
    pub calmar: String,
}

/// Compute all top-bar metrics from a slice of daily candles and an
/// optional live price.
///
/// Each field is a formatted display string.  When the input is empty or a
/// metric cannot be computed the field shows an em-dash `"—"`.
pub fn compute(candles: &[Candle], live_price: Option<f64>) -> Metrics {
    let dash = "\u{2014}".to_string();

    // P/L: first candle close vs live price.
    //  ▲ for positive, ▼ for negative, — for zero.
    let p_l = match (candles.first(), live_price) {
        (Some(first), Some(current)) if first.close != 0.0 => {
            let change = (current - first.close) / first.close * 100.0;
            if change.abs() < 0.001 {
                dash.clone()
            } else if change > 0.0 {
                format!("\u{25B2} {:.2}%", change)
            } else {
                format!("\u{25BC} {:.2}%", -change)
            }
        }
        _ => dash.clone(),
    };

    // High / Low: max of high, min of low across the period.
    fn fmt_price(p: f64) -> String {
        let whole = p.trunc() as i64;
        let cents = ((p - p.trunc()) * 100.0).round() as u64;
        let s = whole.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3 + 4);
        result.push('$');
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result.push('.');
        result.push_str(&format!("{:02}", cents));
        result
    }

    let high = if candles.is_empty() {
        dash.clone()
    } else {
        let max_high = candles.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        fmt_price(max_high)
    };

    let low = if candles.is_empty() {
        dash.clone()
    } else {
        let min_low = candles.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        fmt_price(min_low)
    };

    // Max Drawdown: largest peak-to-trough decline using high/low.
    // Tracks running peak high, then (peak - low) / peak at each candle.
    let draw_down = if candles.is_empty() {
        dash.clone()
    } else {
        let mut peak_high = candles[0].high;
        let mut max_dd = 0.0_f64;
        for c in candles {
            if c.high > peak_high {
                peak_high = c.high;
            }
            let dd = (peak_high - c.low) / peak_high * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        if max_dd < 0.001 {
            dash.clone()
        } else {
            format!("{:.2}%", max_dd)
        }
    };

    // Max Run-up: largest trough-to-peak rise using high/low.
    // Tracks running trough low, then (high - trough) / trough at each candle.
    let run_up = if candles.is_empty() {
        dash.clone()
    } else {
        let mut trough_low = candles[0].low;
        let mut max_ru = 0.0_f64;
        for c in candles {
            if c.low < trough_low {
                trough_low = c.low;
            }
            let ru = (c.high - trough_low) / trough_low * 100.0;
            if ru > max_ru {
                max_ru = ru;
            }
        }
        if max_ru < 0.001 {
            dash.clone()
        } else {
            format!("{:.2}%", max_ru)
        }
    };

    Metrics {
        p_l,
        high,
        low,
        draw_down,
        run_up,
        calmar: dash,
    }
}
