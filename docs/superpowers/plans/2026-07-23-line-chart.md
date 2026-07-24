# Line Chart Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a static line chart widget (Iced Canvas) showing 365 days of BTC close prices with axis labels and crosshair, wired into the YoY dashboard view.

**Architecture:** New `src/modules/ui/line_chart/` module provides the canvas widget and state. New `src/modules/compute/year_over_year.rs` queries the trailing 365 candles from SQLite. The dashboard's price placeholder is replaced with the chart when YoY mode is active.

**Tech Stack:** Iced 0.14.0 Canvas, rusqlite, std::time

---

### Task 1: Create `year_over_year` compute module

**Files:**
- Create: `src/modules/compute/year_over_year.rs`
- Modify: `src/modules/compute/mod.rs`

- [ ] **Step 1: Add module declaration to compute/mod.rs**

Replace:
```rust
pub mod coins_issued;
pub mod halving_eta;
pub mod price_stats;
```
With:
```rust
pub mod coins_issued;
pub mod halving_eta;
pub mod price_stats;
pub mod year_over_year;
```

- [ ] **Step 2: Create year_over_year.rs**

Write to `src/modules/compute/year_over_year.rs`:

```rust
use rusqlite::Connection;
use std::path::PathBuf;

/// A single daily OHLC candle.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Query the trailing 365 days of daily candles from the BTC/USD database.
///
/// Returns candles ordered by timestamp ascending. The range is computed
/// as: `[latest_timestamp - 364 days, latest_timestamp]`.
pub fn trailing_365_candles() -> Vec<Candle> {
    let db_path = db_path();
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[year_over_year] failed to open database: {}", e);
            return Vec::new();
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT timestamp, open, high, low, close, volume
         FROM daily_candles
         WHERE timestamp >= (SELECT MAX(timestamp) FROM daily_candles) - 365 * 86400
         ORDER BY timestamp ASC"
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[year_over_year] failed to prepare query: {}", e);
            return Vec::new();
        }
    };

    let candles: Vec<Candle> = match stmt.query_map([], |row| {
        Ok(Candle {
            timestamp: row.get(0)?,
            open: row.get(1)?,
            high: row.get(2)?,
            low: row.get(3)?,
            close: row.get(4)?,
            volume: row.get(5)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("[year_over_year] failed to query candles: {}", e);
            return Vec::new();
        }
    };

    candles
}

fn db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Halvora").join("Exchange").join("btcusd.db")
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add src/modules/compute/
git commit -m "feat: add year_over_year compute module for 365-day candle query"
```

---

### Task 2: Create `line_chart` module — state and axis math

**Files:**
- Create: `src/modules/ui/line_chart/mod.rs`
- Create: `src/modules/ui/line_chart/state.rs`
- Create: `src/modules/ui/line_chart/axis.rs`
- Modify: `src/modules/ui/mod.rs`

- [ ] **Step 1: Register line_chart module in ui/mod.rs**

Replace:
```rust
pub mod mainwindow;
pub mod theme;
pub mod scaling;
```
With:
```rust
pub mod line_chart;
pub mod mainwindow;
pub mod theme;
pub mod scaling;
```

- [ ] **Step 2: Create state.rs**

Write to `src/modules/ui/line_chart/state.rs`:

```rust
use crate::modules::compute::year_over_year::Candle;

/// Holds all data and viewport state for the line chart.
pub struct LineChartState {
    /// OHLC candles sorted by timestamp ascending.
    pub candles: Vec<Candle>,
    /// Crosshair data-space X (timestamp), set by mouse move.
    pub crosshair_ts: Option<f64>,
    /// Crosshair data-space Y (price), set by mouse move.
    pub crosshair_price: Option<f64>,
}

impl LineChartState {
    /// Create a new state from a list of candles.
    /// Determines viewport bounds automatically from data.
    pub fn new(candles: Vec<Candle>) -> Self {
        Self {
            candles,
            crosshair_ts: None,
            crosshair_price: None,
        }
    }

    /// Compute the visible X range (earliest to latest timestamp).
    pub fn x_bounds(&self) -> (f64, f64) {
        if self.candles.is_empty() {
            return (0.0, 1.0);
        }
        let min = self.candles.first().unwrap().timestamp as f64;
        let max = self.candles.last().unwrap().timestamp as f64;
        if min == max {
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
            if c.close < min { min = c.close; }
            if c.close > max { max = c.close; }
        }
        if min == max {
            return (min - 1.0, max + 1.0);
        }
        let padding = (max - min) * 0.1;
        (min - padding, max + padding)
    }
}
```

- [ ] **Step 3: Create axis.rs**

Write to `src/modules/ui/line_chart/axis.rs`:

```rust
use chrono::{DateTime, Utc, NaiveDateTime};

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
        return vec![
            Tick { position: min, label: format!("${:.0}", min) },
        ];
    }

    // Calculate a "nice" step size
    let raw_step = range / 6.0;
    let magnitude = 10_f64.powf(raw_step.log10().floor());
    let residual = raw_step / magnitude;
    let nice_step = if residual <= 1.5 { magnitude }
        else if residual <= 3.5 { 2.0 * magnitude }
        else if residual <= 7.5 { 5.0 * magnitude }
        else { 10.0 * magnitude };

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
        ticks.push(Tick { position: min, label: format_price(min) });
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
    if min_ts >= max_ts {
        return Vec::new();
    }

    let min_naive = match NaiveDateTime::from_timestamp_opt(min_ts as i64, 0) {
        Some(dt) => dt,
        None => return Vec::new(),
    };
    let max_naive = match NaiveDateTime::from_timestamp_opt(max_ts as i64, 0) {
        Some(dt) => dt,
        None => return Vec::new(),
    };

    let min_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(min_naive, Utc);
    let max_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(max_naive, Utc);

    let mut ticks = Vec::new();
    let mut year = min_dt.year();
    let mut month = min_dt.month();

    loop {
        // Build the first day of this month at 00:00:00 UTC
        let tick_naive = match NaiveDateTime::parse_from_str(
            &format!("{}-{:02}-01 00:00:00", year, month),
            "%Y-%m-%d %H:%M:%S"
        ) {
            Ok(dt) => dt,
            Err(_) => {
                // Advance month
                month += 1;
                if month > 12 {
                    month = 1;
                    year += 1;
                }
                if year > max_dt.year() || (year == max_dt.year() && month > max_dt.month()) {
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
            let label = format!("{} '{}", 
                match tick_dt.month() {
                    1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                    5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                    9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                    _ => "???",
                },
                (tick_dt.year() % 100)
            );
            ticks.push(Tick {
                position: tick_dt.timestamp() as f64,
                label,
            });
        }

        // Advance month
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
        if year > max_dt.year() || (year == max_dt.year() && month > max_dt.month()) {
            break;
        }
    }

    ticks
}
```

Note: This requires adding `chrono` to `Cargo.toml` dependencies. Let's check if it's already there.

- [ ] **Step 4: Add chrono dependency**

Run: `grep -c "chrono" Cargo.toml` — if 0, add it:
```
chrono = "0.4"
```
To `Cargo.toml` under `[dependencies]`.

Since we saw `chrono = "0.4"` in iced_aksel's deps, we know it exists. Let's add it if not present.

Check: `grep chrono Cargo.toml`

If missing, add `chrono = "0.4"` line to `[dependencies]` in Cargo.toml.

- [ ] **Step 5: Create mod.rs**

Write to `src/modules/ui/line_chart/mod.rs`:

```rust
pub mod axis;
pub mod line_chart;
pub mod state;

pub use line_chart::LineChart;
pub use state::LineChartState;
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check 2>&1 | head -30`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add src/modules/ui/line_chart/ src/modules/ui/mod.rs Cargo.toml
git commit -m "feat: add line_chart module with state and axis tick generation"
```

---

### Task 3: Implement the Canvas line chart widget

**Files:**
- Create: `src/modules/ui/line_chart/line_chart.rs`

- [ ] **Step 1: Create line_chart.rs**

Write to `src/modules/ui/line_chart/line_chart.rs`:

```rust
use iced::{
    Color, Element, Event, Length, Point, Rectangle, Size, Vector,
    widget::canvas::{self, Canvas, Cursor, Frame, Path, Stroke, Fill, stroke},
    mouse, Theme,
};
use std::rc::Rc;

use super::axis::{self, Tick};
use super::state::LineChartState;

/// The line chart widget.
pub struct LineChart<'a> {
    state: &'a LineChartState,
    width: Length,
    height: Length,
}

impl<'a> LineChart<'a> {
    pub fn new(state: &'a LineChartState) -> Self {
        Self {
            state,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message> From<LineChart<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(chart: LineChart<'a>) -> Element<'a, Message> {
        Canvas::new(LineChartProgram { state: chart.state })
            .width(chart.width)
            .height(chart.height)
            .into()
    }
}

struct LineChartProgram<'a> {
    state: &'a LineChartState,
}

impl<'a> canvas::Program<Message> for LineChartProgram<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.state.candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let (x_min, x_max) = self.state.x_bounds();
        let (y_min, y_max) = self.state.y_bounds();
        let plot_area = padded_plot_area(bounds);

        // --- 1. Draw grid ---
        draw_grid(&mut frame, &plot_area, x_min, x_max, y_min, y_max);

        // --- 2. Draw price line ---
        draw_price_line(&mut frame, &plot_area, &self.state.candles, x_min, x_max, y_min, y_max);

        // --- 3. Draw axes labels ---
        draw_axes_labels(&mut frame, &plot_area, x_min, x_max, y_min, y_max);

        // --- 4. Draw crosshair ---
        if let (Some(ts), Some(price)) = (self.state.crosshair_ts, self.state.crosshair_price) {
            draw_crosshair(&mut frame, &plot_area, ts, price, x_min, x_max, y_min, y_max);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut (),
        event: Event,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        // We don't emit messages; crosshair is set via the application's update()
        (canvas::event::Status::Ignored, None)
    }
}

/// The plot area with padding for axes.
fn padded_plot_area(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + 60.0,  // left padding for Y-axis labels
        y: bounds.y + 10.0,  // top padding
        width: bounds.width - 70.0,  // right padding
        height: bounds.height - 30.0, // bottom padding for X-axis labels
    }
}

// -- Coordinate conversion helpers --

fn data_x_to_screen(ts: f64, x_min: f64, x_max: f64, plot: &Rectangle) -> f32 {
    let t = (ts - x_min) / (x_max - x_min);
    (plot.x + t * plot.width) as f32
}

fn data_y_to_screen(price: f64, y_min: f64, y_max: f64, plot: &Rectangle) -> f32 {
    let t = (price - y_min) / (y_max - y_min);
    (plot.y + (1.0 - t) * plot.height) as f32
}

fn screen_x_to_data(screen_x: f32, x_min: f64, x_max: f64, plot: &Rectangle) -> f64 {
    let t = (screen_x as f64 - plot.x as f64) / plot.width as f64;
    x_min + t * (x_max - x_min)
}

fn screen_y_to_data(screen_y: f32, y_min: f64, y_max: f64, plot: &Rectangle) -> f64 {
    let t = 1.0 - (screen_y as f64 - plot.y as f64) / plot.height as f64;
    y_min + t * (y_max - y_min)
}

// -- Drawing functions --

const GRID_COLOR: Color = Color::from_rgba(0.3, 0.3, 0.3, 0.15);
const LINE_COLOR: Color = Color::from_rgb(0.0, 0.6, 1.0);
const FILL_COLOR: Color = Color::from_rgba(0.0, 0.6, 1.0, 0.1);
const TEXT_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7);
const CROSSHAIR_COLOR: Color = Color::from_rgba(0.8, 0.8, 0.8, 0.5);

fn draw_grid(frame: &mut Frame, plot: &Rectangle, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
    // Horizontal grid lines (from price ticks)
    let y_ticks = axis::y_ticks(y_min, y_max);
    for tick in &y_ticks {
        let y = data_y_to_screen(tick.position, y_min, y_max, plot);
        let mut path = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(GRID_COLOR)
                .with_width(0.5),
        );
    }

    // Vertical grid lines (from date ticks)
    let x_ticks = axis::x_ticks(x_min, x_max);
    for tick in &x_ticks {
        let x = data_x_to_screen(tick.position, x_min, x_max, plot);
        let mut path = Path::new(|p| {
            p.move_to(Point::new(x, plot.y));
            p.line_to(Point::new(x, plot.y + plot.height));
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(GRID_COLOR)
                .with_width(0.5),
        );
    }
}

fn draw_price_line(frame: &mut Frame, plot: &Rectangle, candles: &[crate::modules::compute::year_over_year::Candle], x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
    if candles.len() < 2 {
        return;
    }

    // Draw the filled area below the line
    let mut fill_path = Path::new(|p| {
        let first_x = data_x_to_screen(candles[0].timestamp as f64, x_min, x_max, plot);
        let first_y = data_y_to_screen(candles[0].close, y_min, y_max, plot);
        p.move_to(Point::new(first_x, plot.y + plot.height));
        p.line_to(Point::new(first_x, first_y));

        for c in &candles[1..] {
            let x = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot);
            let y = data_y_to_screen(c.close, y_min, y_max, plot);
            p.line_to(Point::new(x, y));
        }

        let last_x = data_x_to_screen(candles.last().unwrap().timestamp as f64, x_min, x_max, plot);
        p.line_to(Point::new(last_x, plot.y + plot.height));
        p.close();
    });

    frame.fill(&fill_path, Fill::default().with_color(FILL_COLOR));

    // Draw the line itself
    let mut line_path = Path::new(|p| {
        let first_x = data_x_to_screen(candles[0].timestamp as f64, x_min, x_max, plot);
        let first_y = data_y_to_screen(candles[0].close, y_min, y_max, plot);
        p.move_to(Point::new(first_x, first_y));

        for c in &candles[1..] {
            let x = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot);
            let y = data_y_to_screen(c.close, y_min, y_max, plot);
            p.line_to(Point::new(x, y));
        }
    });

    frame.stroke(
        &line_path,
        Stroke::default()
            .with_color(LINE_COLOR)
            .with_width(2.0),
    );
}

fn draw_axes_labels(frame: &mut Frame, plot: &Rectangle, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
    // Y-axis price labels (left side)
    let y_ticks = axis::y_ticks(y_min, y_max);
    for tick in &y_ticks {
        let y = data_y_to_screen(tick.position, y_min, y_max, plot);
        let text = canvas::Text {
            content: tick.label.clone(),
            position: Point::new(plot.x - 8.0, y),
            color: TEXT_COLOR,
            size: 10.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Right,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        };
        frame.fill_text(text);
    }

    // X-axis date labels (bottom, rotated would be nice but text only)
    let x_ticks = axis::x_ticks(x_min, x_max);
    // Space labels evenly — only show a subset if too many
    let max_labels = 8;
    let step = if x_ticks.len() > max_labels { x_ticks.len() / max_labels } else { 1 };
    for (i, tick) in x_ticks.iter().enumerate() {
        if i % step != 0 && i != x_ticks.len() - 1 {
            continue;
        }
        let x = data_x_to_screen(tick.position, x_min, x_max, plot);
        let text = canvas::Text {
            content: tick.label.clone(),
            position: Point::new(x, plot.y + plot.height + 8.0),
            color: TEXT_COLOR,
            size: 10.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        };
        frame.fill_text(text);
    }
}

fn draw_crosshair(frame: &mut Frame, plot: &Rectangle, ts: f64, price: f64, x_min: f64, x_max: f64, y_min: f64, y_max: f64) {
    let x = data_x_to_screen(ts, x_min, x_max, plot);
    let y = data_y_to_screen(price, y_min, y_max, plot);

    // Vertical line
    if x >= plot.x && x <= plot.x + plot.width {
        let mut vline = Path::new(|p| {
            p.move_to(Point::new(x, plot.y));
            p.line_to(Point::new(x, plot.y + plot.height));
        });
        frame.stroke(
            &vline,
            Stroke::default()
                .with_color(CROSSHAIR_COLOR)
                .with_width(1.0),
        );
    }

    // Horizontal line
    if y >= plot.y && y <= plot.y + plot.height {
        let mut hline = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(
            &hline,
            Stroke::default()
                .with_color(CROSSHAIR_COLOR)
                .with_width(1.0),
        );
    }

    // Price readout (top-right corner)
    let readout = canvas::Text {
        content: format!("${:.2}", price),
        position: Point::new(plot.x + plot.width - 4.0, plot.y + 4.0),
        color: Color::WHITE,
        size: 11.0.into(),
        horizontal_alignment: iced::alignment::Horizontal::Right,
        vertical_alignment: iced::alignment::Vertical::Top,
        ..canvas::Text::default()
    };
    frame.fill_text(readout);
}
```

Note: The `canvas::Program` trait in Iced 0.14 uses `draw()` that returns `Vec<canvas::Geometry>`. Let me make sure we're using the right API.

Actually, in Iced 0.14, the Canvas program trait is:

```rust
pub trait Program<Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    type State: Default + 'static;
    fn draw(&self, state: &Self::State, renderer: &Renderer, theme: &Theme, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry>;
    fn update(&self, state: &mut Self::State, event: Event, bounds: Rectangle, cursor: Cursor) -> (event::Status, Option<Message>);
    ...
}
```

And crosshair needs to be updated via cursor position. Since we said no interaction in the widget itself (just crosshair), the crosshair state should be set from mouse events captured by the parent widget or by listening to cursor events. Actually, the Canvas program receives `cursor: Cursor` in draw and can track mouse position.

Wait — for the crosshair to update, we either need:
1. The application to pass mouse position via state updates
2. The Canvas program to handle `MouseMoved` events in `update()`

Option 2 is better since it doesn't require the parent to relay mouse events. Let me update the widget to handle mouse events internally for the crosshair.

Actually the simplest approach: In the `update` method of the canvas program, listen for `Event::Mouse(mouse::Event::CursorMoved { position })` and store the crosshair position in the program's state. But we need mutable state.

Let me revise line_chart.rs to handle crosshair internally via the program's State:

```rust
struct LineChartProgram<'a> {
    state: &'a LineChartState,
}

impl<'a> canvas::Program<Message> for LineChartProgram<'a> {
    type State = CrosshairState;

    fn update(
        &self,
        state: &mut CrosshairState,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let plot = padded_plot_area(bounds);
                if position.x >= plot.x && position.x <= plot.x + plot.width
                    && position.y >= plot.y && position.y <= plot.y + plot.height
                {
                    let (x_min, x_max) = self.state.x_bounds();
                    let (y_min, y_max) = self.state.y_bounds();
                    state.crosshair_ts = Some(screen_x_to_data(position.x, x_min, x_max, &plot));
                    state.crosshair_price = Some(screen_y_to_data(position.y, y_min, y_max, &plot));
                } else {
                    state.crosshair_ts = None;
                    state.crosshair_price = None;
                }
                (canvas::event::Status::Captured, None)
            }
            Event::Mouse(mouse::Event::CursorLeft) => {
                state.crosshair_ts = None;
                state.crosshair_price = None;
                (canvas::event::Status::Captured, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        state: &CrosshairState,
        ...
    ) {
        ...
        // Use state.crosshair_ts and state.crosshair_price
    }
}
```

Let me also define CrosshairState:

```rust
#[derive(Default)]
struct CrosshairState {
    crosshair_ts: Option<f64>,
    crosshair_price: Option<f64>,
}
```

OK now let me proceed with implementation.Good, I have a clear picture. Let me check if chrono is already in the project dependencies.

<execute_command>
<command>grep -n "chrono" Cargo.toml</command>
<requires_approval>false</requires_approval>
</execute_command>