use crate::modules::compute::year_over_year::Candle;

/// Holds all data and viewport state for the line chart.
pub struct LineChartState {
    /// OHLC candles sorted by timestamp ascending.
    pub candles: Vec<Candle>,
}

impl LineChartState {
    /// Create a new state from a list of candles.
    /// Determines viewport bounds automatically from data.
    pub fn new(candles: Vec<Candle>) -> Self {
        Self { candles }
    }

    /// Compute the visible X range (earliest to latest timestamp).
    pub fn x_bounds(&self) -> (f64, f64) {
        if self.candles.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.candles.first().unwrap().timestamp as f64;
        let max = self.candles.last().unwrap().timestamp as f64;
        if (min - max).abs() < f64::EPSILON {
            (min - 86_400.0, max + 86_400.0)
        } else {
            (min, max)
        }
    }

    /// Compute the visible Y range (min/max close + 10% padding).
    pub fn y_bounds(&self) -> (f64, f64) {
        if self.candles.is_empty() {
            return (0.0, 1.0);
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for c in &self.candles {
            if c.close < min {
                min = c.close;
            }
            if c.close > max {
                max = c.close;
            }
        }
        if (max - min).abs() < f64::EPSILON {
            return (min - 1.0, max + 1.0);
        }
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }
}