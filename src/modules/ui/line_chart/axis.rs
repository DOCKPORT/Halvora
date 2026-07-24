use chrono::{DateTime, Datelike, NaiveDateTime, Utc};

/// A tick mark with its data-space position and formatted label.
#[derive(Debug, Clone)]
pub struct Tick {
    pub position: f64,
    pub label: String,
}

/// Generate Y-axis (price) tick marks.
/// Produces ~6 nice round numbers within [min, max].
pub fn y_ticks(min: f64, max: f64) -> Vec<Tick> {
    let range = max - min;
    if range <= 0.0 {
        return vec![Tick {
            position: min,
            label: format!("${:.0}", min),
        }];
    }

    // Calculate a "nice" step size
    let raw_step = range / 6.0;
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    let residual = raw_step / magnitude;
    let nice_step = if residual <= 1.5 {
        magnitude
    } else if residual <= 3.5 {
        2.0 * magnitude
    } else if residual <= 7.5 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    let start = (min / nice_step).ceil() * nice_step;
    let mut ticks = Vec::new();
    let mut val = start;
    while val <= max {
        ticks.push(Tick {
            position: val,
            label: format_price(val),
        });
        val += nice_step;
    }

    if ticks.is_empty() {
        ticks.push(Tick {
            position: min,
            label: format_price(min),
        });
    }

    ticks
}

/// Format a price value for Y-axis labels.
fn format_price(price: f64) -> String {
    if price >= 1_000_000.0 {
        format!("${:.0}M", price / 1_000_000.0)
    } else if price >= 1_000.0 {
        format!("${:.0}K", price / 1_000.0)
    } else {
        format!("${:.0}", price)
    }
}

/// Generate X-axis (date) tick marks.
/// Produces one tick per month in [min_ts, max_ts].
pub fn x_ticks(min_ts: f64, max_ts: f64) -> Vec<Tick> {
    if min_ts >= max_ts || max_ts - min_ts < 86_400.0 {
        return Vec::new();
    }

    let min_dt = match DateTime::from_timestamp(min_ts as i64, 0) {
        Some(dt) => dt,
        None => return Vec::new(),
    };
    let max_dt = match DateTime::from_timestamp(max_ts as i64, 0) {
        Some(dt) => dt,
        None => return Vec::new(),
    };

    let mut ticks = Vec::new();
    let mut year = min_dt.year();
    let mut month = min_dt.month();

    loop {
        // Build the first day of this month at 00:00:00 UTC
        let tick_str = format!("{}-{:02}-01T00:00:00", year, month);
        let tick_naive = match NaiveDateTime::parse_from_str(&tick_str, "%Y-%m-%dT%H:%M:%S") {
            Ok(dt) => dt,
            Err(_) => {
                advance_month(&mut year, &mut month);
                if past_end(year, month, &max_dt) {
                    break;
                }
                continue;
            }
        };
        let tick_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(tick_naive, Utc);

        if tick_dt > max_dt {
            break;
        }

        if tick_dt >= min_dt {
            let label = format!(
                "{} '{}",
                match tick_dt.month() {
                    1 => "Jan",
                    2 => "Feb",
                    3 => "Mar",
                    4 => "Apr",
                    5 => "May",
                    6 => "Jun",
                    7 => "Jul",
                    8 => "Aug",
                    9 => "Sep",
                    10 => "Oct",
                    11 => "Nov",
                    12 => "Dec",
                    _ => "???",
                },
                tick_dt.year() % 100
            );
            ticks.push(Tick {
                position: tick_dt.timestamp() as f64,
                label,
            });
        }

        advance_month(&mut year, &mut month);
        if past_end(year, month, &max_dt) {
            break;
        }
    }

    ticks
}

fn advance_month(year: &mut i32, month: &mut u32) {
    *month += 1;
    if *month > 12 {
        *month = 1;
        *year += 1;
    }
}

fn past_end(year: i32, month: u32, max_dt: &DateTime<Utc>) -> bool {
    year > max_dt.year() || (year == max_dt.year() && month > max_dt.month())
}