use std::cell::{Cell, RefCell};
use crate::modules::compute::year_over_year::Candle;

/// Holds all data and viewport state for the line chart.
pub struct LineChartState {
    /// OHLC candles sorted by timestamp ascending.
    pub candles: Vec<Candle>,
    /// Crosshair hover index shared with the volume chart.
    /// `None` when the cursor is outside the plot area.
    pub hovered_index: Cell<Option<usize>>,
    /// Anchored VWAP start indices (candle indices in insertion order).
    /// Users left-click to add, right-click near a line to remove.
    pub anchored_vwaps: RefCell<Vec<usize>>,
}

impl LineChartState {
    /// Create a new state from a list of candles.
    /// Determines viewport bounds automatically from data.
    pub fn new(candles: Vec<Candle>) -> Self {
        Self {
            candles,
            hovered_index: Cell::new(None),
            anchored_vwaps: RefCell::new(Vec::new()),
        }
    }

    /// Add an anchored VWAP starting at the given candle index.
    /// Duplicates are ignored — the same candle cannot anchor more than once.
    pub fn push_anchor(&self, idx: usize) {
        let mut anchors = self.anchored_vwaps.borrow_mut();
        if !anchors.contains(&idx) {
            anchors.push(idx);
        }
    }

    /// Remove the anchor at `list_idx` (position in the anchor list, not candle index).
    pub fn remove_anchor_at(&self, list_idx: usize) {
        self.anchored_vwaps.borrow_mut().remove(list_idx);
    }

    /// Return a snapshot of all anchor candle indices.
    pub fn anchors(&self) -> Vec<usize> {
        self.anchored_vwaps.borrow().clone()
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