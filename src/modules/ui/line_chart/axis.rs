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
            label: format!("${min:.0}"),
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
        format!("${price:.0}")
    }
}

/// Generate X-axis (date) tick marks.
/// Produces one tick per month in [`min_ts`, `max_ts`].
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
        let tick_str = format!("{year}-{month:02}-01T00:00:00");
        let tick_naive = if let Ok(dt) = NaiveDateTime::parse_from_str(&tick_str, "%Y-%m-%dT%H:%M:%S") { dt } else {
            advance_month(&mut year, &mut month);
            if past_end(year, month, &max_dt) {
                break;
            }
            continue;
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

/// A quarter boundary: its timestamp and the quarter number (1..4).
struct QuarterBoundary {
    position: f64,
    quarter: u32,
    year: i32,
}

/// Enumerate the quarter-start boundaries (Jan 1, Apr 1, Jul 1, Oct 1)
/// within [`min_ts`, `max_ts`].
fn quarter_boundaries(min_ts: f64, max_ts: f64) -> Vec<QuarterBoundary> {
    if min_ts >= max_ts {
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

    let quarter_months = [1u32, 4, 7, 10];
    let mut boundaries = Vec::new();

    // Walk through all quarter boundaries in the range
    let mut year = min_dt.year();
    let mut mi = match min_dt.month() {
        1..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    };

    loop {
        let qm = quarter_months[mi];
        if year > max_dt.year() || (year == max_dt.year() && qm > max_dt.month()) {
            break;
        }

        let tick_str = format!("{year:04}-{qm:02}-01T00:00:00");
        if let Ok(tick_naive) = NaiveDateTime::parse_from_str(&tick_str, "%Y-%m-%dT%H:%M:%S") {
            let tick_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(tick_naive, Utc);
            if tick_dt >= min_dt && tick_dt <= max_dt {
                boundaries.push(QuarterBoundary {
                    position: tick_dt.timestamp() as f64,
                    quarter: (qm / 3) + 1,
                    year,
                });
            }
        }

        mi += 1;
        if mi >= 4 {
            mi = 0;
            year += 1;
        }
    }

    boundaries
}

/// Generate quarter boundary tick marks (Q1, Q2, Q3, Q4).
/// Returns ticks for quarter-start months (Jan 1, Apr 1, Jul 1, Oct 1).
pub fn quarter_ticks(min_ts: f64, max_ts: f64) -> Vec<Tick> {
    quarter_boundaries(min_ts, max_ts)
        .into_iter()
        .map(|b| Tick {
            position: b.position,
            label: format!("Q{}", b.quarter),
        })
        .collect()
}

/// Generate quarter-start month labels, for example "JAN '26".
/// Returns ticks at the same positions as `quarter_ticks`, but each label is
/// the starting month of that quarter plus the short year.
pub fn quarter_month_ticks(min_ts: f64, max_ts: f64) -> Vec<Tick> {
    quarter_boundaries(min_ts, max_ts)
        .into_iter()
        .map(|b| Tick {
            position: b.position,
            label: format!("{} '{}", month_abbr(quarter_month(b.quarter)), b.year % 100),
        })
        .collect()
}

/// The starting month (1-based) of a quarter.
fn quarter_month(quarter: u32) -> u32 {
    match quarter {
        1 => 1,
        2 => 4,
        3 => 7,
        _ => 10,
    }
}

/// Uppercase three-letter month abbreviation.
fn month_abbr(month: u32) -> &'static str {
    match month {
        1 => "JAN",
        2 => "FEB",
        3 => "MAR",
        4 => "APR",
        5 => "MAY",
        6 => "JUN",
        7 => "JUL",
        8 => "AUG",
        9 => "SEP",
        10 => "OCT",
        11 => "NOV",
        12 => "DEC",
        _ => "???",
    }
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
