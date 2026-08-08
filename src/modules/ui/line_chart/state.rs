use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::ws_flash::WsFlash;
use std::cell::{Cell, RefCell};

/// Which drawing tool is currently active on the chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawingMode {
    /// Anchored VWAP lines — left-click to anchor, right-click to remove.
    #[default]
    AVWAP,
    /// Range selection tool (future).
    Range,
}

/// A completed range annotation on the chart.
#[derive(Debug, Clone, Copy)]
pub struct RangeBox {
    pub from_ts: f64,
    pub from_price: f64,
    pub to_ts: f64,
    pub to_price: f64,
}

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
    /// When true, the chart ignores mouse events (e.g. a modal is open).
    pub dialog_open: Cell<bool>,
    /// Which drawing tool is currently selected.
    pub drawing_mode: Cell<DrawingMode>,
    /// Completed range boxes drawn on the chart.
    pub ranges: RefCell<Vec<RangeBox>>,
    /// During range placement: (`from_ts`, `from_price`) after first click, then
    /// `None` when placement concludes or is cancelled.
    pub range_pending: Cell<Option<(f64, f64)>>,
    /// During range placement: current cursor position for live preview.
    pub range_preview: Cell<Option<(f64, f64)>>,
    /// Active websocket price flash (green up / red down), mirrored from the
    /// application so the chart tooltip can highlight the changed digits.
    pub ws_flash: Cell<Option<WsFlash>>,
    /// Block range shown in the top-left for a started (past or current)
    /// halving, as `(start_height, next_start_height)`. `None` for YOY or when
    /// no halving is active.
    pub block_range: Cell<Option<(u64, u64)>>,
}

impl LineChartState {
    /// Create a new state from a list of candles.
    /// Determines viewport bounds automatically from data.
    pub fn new(candles: Vec<Candle>) -> Self {
        Self {
            candles,
            hovered_index: Cell::new(None),
            anchored_vwaps: RefCell::new(Vec::new()),
            dialog_open: Cell::new(false),
            drawing_mode: Cell::new(DrawingMode::default()),
            ranges: RefCell::new(Vec::new()),
            range_pending: Cell::new(None),
            range_preview: Cell::new(None),
            ws_flash: Cell::new(None),
            block_range: Cell::new(None),
        }
    }

    /// Replace the candle data in place.
    ///
    /// Does not reset the drawing tool mode, anchored VWAPs, or range
    /// annotations, so user drawings survive a data change. Use this instead
    /// of `new()` when only the underlying data changes.
    pub fn set_candles(&mut self, candles: Vec<Candle>) {
        self.candles = candles;
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
