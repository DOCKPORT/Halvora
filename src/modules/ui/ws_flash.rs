use iced::Color;
use std::time::{Duration, Instant};

/// How long the flash color stays visible on the value after a websocket tick.
pub const FLASH_DURATION: Duration = Duration::from_secs(1);

/// Solid color used when the price moves up, matching the chart's green.
pub const FLASH_UP: Color = Color::from_rgb(0.0, 0.8, 0.3);
/// Solid color used when the price moves down, matching the chart's red.
pub const FLASH_DOWN: Color = Color::from_rgb(1.0, 0.1, 0.05);

/// Which direction the price moved on the last websocket tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlashDirection {
    /// The price moved up — flash green.
    Up,
    /// The price moved down — flash red.
    Down,
}

/// A single websocket flash: the movement direction, the previous and current
/// prices (to know which digits changed), plus when it started so callers can
/// render the color and let it expire after `FLASH_DURATION`.
#[derive(Debug, Clone, Copy)]
pub struct WsFlash {
    direction: FlashDirection,
    prev: f64,
    next: f64,
    started: Instant,
}

impl WsFlash {
    /// Build a flash from the previous price to the new tick price.
    ///
    /// Returns `None` when there is no previous price (first tick) or the
    /// price did not change, so nothing flashes.
    pub fn from_tick(prev: Option<f64>, next: f64) -> Option<Self> {
        let direction = match prev {
            Some(p) if next > p => FlashDirection::Up,
            Some(p) if next < p => FlashDirection::Down,
            _ => return None,
        };
        Some(Self {
            direction,
            prev: prev.unwrap(),
            next,
            started: Instant::now(),
        })
    }

    /// Whether the flash is still visible at `now` (elapsed < `FLASH_DURATION`).
    pub fn is_active(&self, now: Instant) -> bool {
        now.duration_since(self.started) <= FLASH_DURATION
    }

    /// The color to render the changed digits while this flash is active.
    pub fn color(&self) -> Color {
        match self.direction {
            FlashDirection::Up => FLASH_UP,
            FlashDirection::Down => FLASH_DOWN,
        }
    }

    /// Byte offset in `format_usd(next)` where the changed suffix begins — the
    /// leftmost character that differs from `format_usd(prev)`.
    ///
    /// Everything from this offset to the end of the current value flashes.
    /// Returns `None` when both values format identically.
    pub fn diff_index(&self) -> Option<usize> {
        diff_index_between(self.prev, self.next)
    }

    /// Byte offset where the changed suffix begins when comparing the flash's
    /// previous price against an arbitrary current price. Used by renderers
    /// (e.g. the chart tooltip) that display a value other than `next`.
    pub fn diff_vs(&self, current: f64) -> Option<usize> {
        diff_index_between(self.prev, current)
    }
}

/// Byte offset in `format_usd(next)` where the changed suffix begins — the
/// leftmost character that differs from `format_usd(prev)`.
///
/// Everything from this offset to the end of the current value flashes.
/// Returns `None` when both values format identically.
pub fn diff_index_between(prev: f64, next: f64) -> Option<usize> {
    let a = format_usd(prev);
    let b = format_usd(next);
    let n = a.len().min(b.len());
    for i in 0..n {
        if a.as_bytes()[i] != b.as_bytes()[i] {
            return Some(i);
        }
    }
    if a.len() == b.len() { None } else { Some(n) }
}

/// Format a price with thousands separators and two decimals, e.g.
/// `64000.0` -> `"64,000.00"`. Rounds to whole cents first to avoid drift.
pub fn format_usd(p: f64) -> String {
    fn fmt_commas(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(c);
        }
        result
    }
    let cents = (p * 100.0).round() as i64;
    let whole = fmt_commas((cents / 100) as u64);
    let frac = (cents % 100).abs();
    format!("{whole}.{frac:02}")
}
