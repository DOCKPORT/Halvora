/// A tick mark with its data-space position and formatted label.
#[derive(Debug, Clone)]
pub struct Tick {
    pub position: f64,
    pub label: String,
}

/// Generate Y-axis volume tick marks.
/// Produces ticks at 33% and 67% of the range, formatted with human-readable
/// volume suffixes (e.g. "1.5M").
pub fn y_ticks(y_min: f64, y_max: f64) -> Vec<Tick> {
    let range = y_max - y_min;
    if range <= 0.0 {
        return Vec::new();
    }
    let lower_pos = y_min + range * 0.3333;
    let upper_pos = y_min + range * 0.6667;
    vec![
        Tick {
            position: lower_pos,
            label: fmt_volume(lower_pos),
        },
        Tick {
            position: upper_pos,
            label: fmt_volume(upper_pos),
        },
    ]
}

fn fmt_volume(v: f64) -> String {
    if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.0}K", v / 1_000.0)
    } else {
        format!("{v:.0}")
    }
}
