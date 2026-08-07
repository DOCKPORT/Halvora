use crate::modules::compute::year_over_year::Candle;

/// P/L sign of a period, used to color the sidebar buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLSign {
    Positive,
    Negative,
    NoChange,
}

/// Intermediate values used to compute the Calmar ratio, shown in the detail dialog.
#[derive(Debug, Clone)]
pub struct CalmarBreakdown {
    pub weighted_avg_pl: String,
    pub annualized_return: String,
    pub max_drawdown: String,
    pub ratio: String,
}

/// Display-ready values for the top metric cards.
#[derive(Debug, Clone)]
pub struct Metrics {
    pub p_l: String,
    pub high: String,
    pub low: String,
    pub draw_down: String,
    pub run_up: String,
    pub calmar: String,
    pub calmar_breakdown: CalmarBreakdown,
}

/// Sign of the P/L for a period: first candle close vs the current price.
///
/// Mirrors the math used by the top-bar P/L card so the sidebar buttons stay
/// in sync. Returns `NoChange` when the input is empty, the first close is
/// zero, or the change is below the display threshold.
pub fn pl_sign(candles: &[Candle], current_price: Option<f64>) -> PLSign {
    match (candles.first(), current_price) {
        (Some(first), Some(current)) if first.close != 0.0 => {
            let change = (current - first.close) / first.close * 100.0;
            if change.abs() < 0.001 {
                PLSign::NoChange
            } else if change > 0.0 {
                PLSign::Positive
            } else {
                PLSign::Negative
            }
        }
        _ => PLSign::NoChange,
    }
}

/// Compute all top-bar metrics from a slice of daily candles and an
/// optional live price.
///
/// Each field is a formatted display string.  When the input is empty or a
/// metric cannot be computed the field shows an em-dash `"—"`.
pub fn compute(candles: &[Candle], live_price: Option<f64>) -> Metrics {
    let dash = "\u{2014}".to_string();

    // P/L: first candle close vs live price.
    // The match on `pl_sign` guarantees that when the sign is Positive or
    // Negative, both `live_price` and `candles.first()` are `Some` and the
    // first close is non-zero, so computing `change` up front is safe.
    //  ▲ for positive, ▼ for negative, — for zero.
    let change = if matches!(pl_sign(candles, live_price), PLSign::Positive | PLSign::Negative)
    {
        (live_price.unwrap() - candles.first().unwrap().close) / candles.first().unwrap().close
            * 100.0
    } else {
        0.0
    };
    let p_l = match pl_sign(candles, live_price) {
        PLSign::Positive => format!("\u{25B2} {change:.2}%"),
        PLSign::Negative => format!("\u{25BC} {:.2}%", -change),
        PLSign::NoChange => dash.clone(),
    };

    // High / Low: max of high, min of low across the period.
    fn fmt_price(p: f64) -> String {
        let whole = p.trunc() as i64;
        let cents = ((p - p.trunc()) * 100.0).round() as u64;
        let s = whole.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3 + 4);
        result.push('$');
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(c);
        }
        result.push('.');
        result.push_str(&format!("{cents:02}"));
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
    // Computed as a float (percentage) first, so it can be reused by Calmar.
    let max_dd = if candles.is_empty() {
        0.0
    } else {
        let mut peak_high = candles[0].high;
        let mut dd = 0.0_f64;
        for c in candles {
            if c.high > peak_high {
                peak_high = c.high;
            }
            let decline = (peak_high - c.low) / peak_high * 100.0;
            if decline > dd {
                dd = decline;
            }
        }
        dd
    };

    let draw_down = if max_dd < 0.001 {
        dash.clone()
    } else {
        format!("{max_dd:.2}%")
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
            format!("{max_ru:.2}%")
        }
    };

    // Calmar Ratio:
    //   1. Daily P/L% = (close - open) / open for each candle
    //   2. Volume-weighted average of daily returns
    //   3. Annualize: × 365
    //   4. Ratio = annualized return / (max_drawdown / 100)
    let (calmar, calmar_breakdown) = if candles.is_empty() || max_dd < 0.001 {
        (
            dash.clone(),
            CalmarBreakdown {
                weighted_avg_pl: dash.clone(),
                annualized_return: dash.clone(),
                max_drawdown: dash.clone(),
                ratio: dash.clone(),
            },
        )
    } else {
        let mut weighted_sum = 0.0_f64;
        let mut total_vol = 0.0_f64;
        for c in candles {
            if c.open != 0.0 {
                let daily_pl = (c.close - c.open) / c.open; // decimal
                weighted_sum += daily_pl * c.volume;
                total_vol += c.volume;
            }
        }
        if total_vol <= 0.0 {
            (
                dash.clone(),
                CalmarBreakdown {
                    weighted_avg_pl: dash.clone(),
                    annualized_return: dash.clone(),
                    max_drawdown: dash.clone(),
                    ratio: dash.clone(),
                },
            )
        } else {
            let avg_daily_return = weighted_sum / total_vol;
            let annualized = avg_daily_return * 365.0;
            let ratio = annualized / (max_dd / 100.0);
            let calmar_str = format!("{ratio:.2}");
            (
                calmar_str.clone(),
                CalmarBreakdown {
                    weighted_avg_pl: format!("{:.4}%", avg_daily_return * 100.0),
                    annualized_return: format!("{:.2}%", annualized * 100.0),
                    max_drawdown: format!("{max_dd:.2}%"),
                    ratio: calmar_str,
                },
            )
        }
    };

    Metrics {
        p_l,
        high,
        low,
        draw_down,
        run_up,
        calmar,
        calmar_breakdown,
    }
}
