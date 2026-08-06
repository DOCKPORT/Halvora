use chrono::{DateTime, Datelike};
use iced::keyboard;
use iced::keyboard::key;
use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Fill, Frame, Geometry, Path, Stroke, Style,
};
use iced::widget::text;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use super::axis;
use super::state::LineChartState;
use crate::modules::compute::vwap::progressive_vwap;
use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::mainwindow::dashboard_layout::drawing_tools;
use crate::modules::ui::scaling::sp;

/// Crosshair state tracked internally by the canvas program on mouse move.
#[derive(Default, Clone, Copy)]
struct CrosshairState {
    candle: Option<Candle>,
    active_idx: Option<usize>,
}

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

    #[allow(dead_code)]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[allow(dead_code)]
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
        Canvas::new(LineChartProgram {
            data: chart.state,
        })
        .width(chart.width)
        .height(chart.height)
        .into()
    }
}

struct LineChartProgram<'a> {
    data: &'a LineChartState,
}

impl<Message> canvas::Program<Message> for LineChartProgram<'_> {
    type State = CrosshairState;

    fn draw(
        &self,
        state: &CrosshairState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.data.candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let (x_min, x_max) = self.data.x_bounds();
        let (y_min, y_max) = self.data.y_bounds();
        let plot = padded_plot_area(bounds);

        // 1. Grid
        draw_grid(&mut frame, &plot, x_min, x_max, y_min, y_max);

        // 2. Grey border around the plot area (inside axis labels)
        let border_path = Path::new(|p| {
            p.move_to(Point::new(plot.x, plot.y));
            p.line_to(Point::new(plot.x + plot.width, plot.y));
            p.line_to(Point::new(plot.x + plot.width, plot.y + plot.height));
            p.line_to(Point::new(plot.x, plot.y + plot.height));
            p.close();
        });
        frame.stroke(
            &border_path,
            Stroke::default()
                .with_color(Color::from_rgb(0.4, 0.4, 0.4))
                .with_width(1.0),
        );

        // 3. Quarter boundary lines (behind price content)
        draw_quarter_lines(&mut frame, &plot, x_min, x_max);

        // 4. Price line with fill
        draw_price_line(
            &mut frame,
            &plot,
            &self.data.candles,
            x_min,
            x_max,
            y_min,
            y_max,
        );

        // 5. Progressive VWAP line (white, on top of price line)
        draw_vwap_line(
            &mut frame,
            &plot,
            &self.data.candles,
            x_min,
            x_max,
            y_min,
            y_max,
        );

        // 6. Anchored VWAP lines (also white, same style)
        drawing_tools::draw_anchored_vwaps(
            &mut frame,
            &plot,
            &self.data.candles,
            x_min,
            x_max,
            y_min,
            y_max,
            &self.data.anchors(),
        );

        // 6. Axes labels
        draw_axes_labels(&mut frame, &plot, x_min, x_max, y_min, y_max);

        // 7. Crosshair — hovered candle or today's candle as default
        let flash = self
            .data
            .ws_flash
            .get()
            .filter(|f| f.is_active(std::time::Instant::now()));
        if let (Some(candle), Some(active_idx)) = (&state.candle, state.active_idx) {
            draw_crosshair(
                &mut frame, &plot,
                candle, active_idx,
                &self.data.candles, x_min, x_max,
                true, // show vertical line
                flash,
            );
        } else if let Some((today_cdl, today_idx)) = today_candle(&self.data.candles)
            .or_else(|| last_candle(&self.data.candles))
        {
            draw_crosshair(
                &mut frame, &plot,
                &today_cdl, today_idx,
                &self.data.candles, x_min, x_max,
                false, // no vertical line — not hovered
                flash,
            );
        }

        // 8. Range boxes (completed + preview) — drawn on top of everything
        drawing_tools::draw_ranges(
            &mut frame, &plot,
            x_min, x_max, y_min, y_max,
            self.data,
        );

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut CrosshairState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // When a modal dialog is open, ignore all mouse events on the canvas.
        if self.data.dialog_open.get() {
            return None;
        }
        // Right-click is a global delete: remove whichever drawing is under the
        // cursor (a range box or an anchored VWAP line) no matter which tool is
        // currently selected.
        if let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            use crate::modules::ui::mainwindow::dashboard_layout::drawing_tools::RangeActionResult;
            return match drawing_tools::handle_right_click_delete(cursor, bounds, self.data) {
                RangeActionResult::RedrawAndCapture => {
                    Some(canvas::Action::request_redraw().and_capture())
                }
                RangeActionResult::Redraw => Some(canvas::Action::request_redraw()),
                RangeActionResult::None => None,
            };
        }
        // In Range mode, process clicks for range placement but let cursor
        // events fall through so crosshair tracking still works.
        if self.data.drawing_mode.get() == crate::modules::ui::line_chart::state::DrawingMode::Range {
            use crate::modules::ui::mainwindow::dashboard_layout::drawing_tools::RangeActionResult;
            if let canvas::Event::Mouse(mouse::Event::ButtonPressed(_)) = event {
                return match drawing_tools::handle_range_event(event, bounds, cursor, self.data) {
                    RangeActionResult::RedrawAndCapture => {
                        Some(canvas::Action::request_redraw().and_capture())
                    }
                    RangeActionResult::Redraw => {
                        Some(canvas::Action::request_redraw())
                    }
                    RangeActionResult::None => None,
                };
            }
            // Also handle CursorMoved for live preview update
            if let canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
                match drawing_tools::handle_range_event(event, bounds, cursor, self.data) {
                    RangeActionResult::Redraw => {
                        // Still need to update crosshair too, so fall through
                    }
                    _ => {}
                }
            }
        }

        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    // Ignore clicks on a page with no data.
                    if self.data.candles.is_empty() {
                        state.active_idx = None;
                        return None;
                    }
                    if let Some(idx) = state.active_idx {
                        self.data.push_anchor(idx);
                        return Some(canvas::Action::request_redraw().and_capture());
                    }
                    None
                }
                mouse::Event::CursorMoved { position } => {
                    // In update(), bounds are screen-absolute and cursor
                    // is screen-absolute, so we must keep bounds.x/y here.
                    let plot = Rectangle {
                        x: bounds.x + 60.0,
                        y: bounds.y + 60.0,
                        width: bounds.width - 120.0,
                        height: bounds.height - 120.0,
                    };
                    // A page with no data (e.g. a future halving) renders the
                    // chart with an empty candle set. Guard here so we never
                    // index into an empty slice.
                    if self.data.candles.is_empty() {
                        state.candle = None;
                        state.active_idx = None;
                        self.data.hovered_index.set(None);
                        return Some(canvas::Action::request_redraw().and_capture());
                    }
                    if position.x >= plot.x
                        && position.x <= plot.x + plot.width
                        && position.y >= plot.y
                        && position.y <= plot.y + plot.height
                    {
                        let (x_min, x_max) = self.data.x_bounds();
                        let cursor_ts = screen_x_to_data(position.x, x_min, x_max, &plot);

                        // Binary search to find nearest candle
                        let candles = &self.data.candles;
                        let idx = candles.partition_point(|c| (c.timestamp as f64) < cursor_ts);
                        let nearest_idx = if idx == 0 {
                            0
                        } else if idx >= candles.len() {
                            candles.len() - 1
                        } else {
                            let left_dist = cursor_ts - candles[idx - 1].timestamp as f64;
                            let right_dist = candles[idx].timestamp as f64 - cursor_ts;
                            if left_dist <= right_dist { idx - 1 } else { idx }
                        };
                        state.candle = Some(candles[nearest_idx]);
                        state.active_idx = Some(nearest_idx);
                        self.data.hovered_index.set(Some(nearest_idx));
                    } else {
                        state.candle = None;
                        state.active_idx = None;
                        self.data.hovered_index.set(None);
                    }
                    Some(canvas::Action::request_redraw().and_capture())
                }
                mouse::Event::CursorLeft => {
                    state.candle = None;
                    state.active_idx = None;
                    self.data.hovered_index.set(None);
                    Some(canvas::Action::request_redraw().and_capture())
                }
                _ => None,
            },
            canvas::Event::Keyboard(key_event) => match key_event {
                keyboard::Event::KeyPressed { key: k, .. } => {
                    let candles = &self.data.candles;
                    if candles.is_empty() {
                        return None;
                    }
                    let idx = state.active_idx.unwrap_or(candles.len() - 1);
                    let new_idx = match k.as_ref() {
                        key::Key::Named(key::Named::ArrowLeft) => idx.saturating_sub(1),
                        key::Key::Named(key::Named::ArrowRight) => {
                            if idx + 1 < candles.len() { idx + 1 } else { idx }
                        }
                        _ => return None,
                    };
                    if new_idx != idx {
                        state.active_idx = Some(new_idx);
                        state.candle = Some(candles[new_idx]);
                        self.data.hovered_index.set(Some(new_idx));
                        Some(canvas::Action::request_redraw().and_capture())
                    } else {
                        None
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

// ── Layout ───────────────────────────────────────────────────────────────

/// The plot area with padding for axes.
///
/// In `draw()`, the Frame origin is (0,0) at the widget's top-left,
/// so we use 0.0 as the base coordinate. In `update()`, bounds are
/// screen-absolute — the caller offsets before calling.
fn padded_plot_area(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: 60.0,
        y: 60.0,
        width: bounds.width - 120.0,
        height: bounds.height - 120.0,
    }
}

// ── Coordinate conversion ────────────────────────────────────────────────

fn data_x_to_screen(ts: f64, x_min: f64, x_max: f64, plot: &Rectangle) -> f32 {
    let t = (ts - x_min) / (x_max - x_min);
    plot.x + (t as f32) * plot.width
}

fn data_y_to_screen(price: f64, y_min: f64, y_max: f64, plot: &Rectangle) -> f32 {
    let t = (price - y_min) / (y_max - y_min);
    plot.y + (1.0 - t as f32) * plot.height
}

fn screen_x_to_data(screen_x: f32, x_min: f64, x_max: f64, plot: &Rectangle) -> f64 {
    let t = (screen_x - plot.x) / plot.width;
    x_min + (t as f64) * (x_max - x_min)
}

fn screen_y_to_data(screen_y: f32, y_min: f64, y_max: f64, plot: &Rectangle) -> f64 {
    let t = 1.0 - (screen_y - plot.y) / plot.height;
    y_min + (t as f64) * (y_max - y_min)
}

// ── Colour constants ─────────────────────────────────────────────────────

const GRID_COLOR: Color = Color::from_rgba(0.3, 0.3, 0.3, 0.35);
const GREEN_LINE: Color = Color::from_rgb(0.0, 0.8, 0.3);
const GREEN_FILL: Color = Color::from_rgba(0.0, 0.8, 0.3, 0.1);
const RED_LINE: Color = Color::from_rgb(1.0, 0.1, 0.05);
const RED_FILL: Color = Color::from_rgba(1.0, 0.1, 0.05, 0.1);
const GREY_LINE: Color = Color::from_rgb(0.5, 0.5, 0.5);
const GREY_FILL: Color = Color::from_rgba(0.5, 0.5, 0.5, 0.1);
const TEXT_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7);
const CROSSHAIR_COLOR: Color = Color::from_rgba(0.8, 0.8, 0.8, 0.5);
const VWAP_COLOR: Color = Color::WHITE;

/// Determine the line and fill colour based on price trend.
/// Up: green, Down: red, Flat: grey.
fn trend_colours(candles: &[Candle]) -> (Color, Color) {
    if candles.len() < 2 {
        return (GREY_LINE, GREY_FILL);
    }
    let first = candles.first().unwrap().close;
    let last = candles.last().unwrap().close;
    let diff = last - first;
    if diff.abs() < 0.001 {
        (GREY_LINE, GREY_FILL)
    } else if diff > 0.0 {
        (GREEN_LINE, GREEN_FILL)
    } else {
        (RED_LINE, RED_FILL)
    }
}

// ── Drawing helpers ──────────────────────────────────────────────────────

fn draw_grid(
    frame: &mut Frame,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    // Horizontal lines (from price ticks)
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        let path = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(&path, Stroke::default().with_color(GRID_COLOR).with_width(0.5));
    }

    // Vertical lines (from date ticks)
    let ticks = axis::x_ticks(x_min, x_max);
    for t in &ticks {
        let x = data_x_to_screen(t.position, x_min, x_max, plot);
        let path = Path::new(|p| {
            p.move_to(Point::new(x, plot.y));
            p.line_to(Point::new(x, plot.y + plot.height));
        });
        frame.stroke(&path, Stroke::default().with_color(GRID_COLOR).with_width(0.5));
    }
}

fn draw_price_line(
    frame: &mut Frame,
    plot: &Rectangle,
    candles: &[Candle],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    if candles.len() < 2 {
        return;
    }

    let (line_color, fill_color) = trend_colours(candles);

    // Gradient fill below the line
    let fill_path = Path::new(|p| {
        let first_x = data_x_to_screen(candles[0].timestamp as f64, x_min, x_max, plot);
        let first_y = data_y_to_screen(candles[0].close, y_min, y_max, plot);
        p.move_to(Point::new(first_x, plot.y + plot.height));
        p.line_to(Point::new(first_x, first_y));

        for c in &candles[1..] {
            let x = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot);
            let y = data_y_to_screen(c.close, y_min, y_max, plot);
            p.line_to(Point::new(x, y));
        }

        let last_x =
            data_x_to_screen(candles.last().unwrap().timestamp as f64, x_min, x_max, plot);
        p.line_to(Point::new(last_x, plot.y + plot.height));
        p.close();
    });
    frame.fill(&fill_path, Fill {
        style: Style::Solid(fill_color),
        ..Fill::default()
    });

    // The line itself
    let line_path = Path::new(|p| {
        let first_x = data_x_to_screen(candles[0].timestamp as f64, x_min, x_max, plot);
        let first_y = data_y_to_screen(candles[0].close, y_min, y_max, plot);
        p.move_to(Point::new(first_x, first_y));

        for c in &candles[1..] {
            let x = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot);
            let y = data_y_to_screen(c.close, y_min, y_max, plot);
            p.line_to(Point::new(x, y));
        }
    });
    frame.stroke(&line_path, Stroke::default().with_color(line_color).with_width(2.0));
}

/// Draw the progressive cumulative VWAP line (white, 2px).
///
/// Computes VWAP at each candle using all data from the first candle
/// up to that candle, then plots the path.  Entries with zero cumulative
/// volume are skipped.
fn draw_vwap_line(
    frame: &mut Frame,
    plot: &Rectangle,
    candles: &[Candle],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    let pairs: Vec<(f64, f64)> = candles
        .iter()
        .map(|c| (c.close, c.volume))
        .collect();

    let vwaps = progressive_vwap(&pairs);

    // Collect (timestamp, vwap) for points that are Some
    let points: Vec<(f64, f64)> = candles
        .iter()
        .zip(vwaps.iter())
        .filter_map(|(c, v)| v.map(|vwap| (c.timestamp as f64, vwap)))
        .collect();

    if points.len() < 2 {
        return;
    }

    let path = Path::new(|p| {
        let first_x = data_x_to_screen(points[0].0, x_min, x_max, plot);
        let first_y = data_y_to_screen(points[0].1, y_min, y_max, plot);
        p.move_to(Point::new(first_x, first_y));

        for &(ts, vwap) in &points[1..] {
            let x = data_x_to_screen(ts, x_min, x_max, plot);
            let y = data_y_to_screen(vwap, y_min, y_max, plot);
            p.line_to(Point::new(x, y));
        }
    });

    frame.stroke(&path, Stroke::default().with_color(VWAP_COLOR).with_width(1.5));
}

/// Draw subtle vertical lines at quarter boundaries (behind price content).
fn draw_quarter_lines(frame: &mut Frame, plot: &Rectangle, x_min: f64, x_max: f64) {
    let qcolor = Color::from_rgba(0.6, 0.6, 0.6, 0.3);
    for q in axis::quarter_ticks(x_min, x_max) {
        let x = data_x_to_screen(q.position, x_min, x_max, plot);
        if x < plot.x || x > plot.x + plot.width {
            continue;
        }
        let path = Path::new(|p| {
            p.move_to(Point::new(x, plot.y));
            p.line_to(Point::new(x, plot.y + plot.height));
        });
        frame.stroke(
            &path,
            Stroke::default().with_color(qcolor).with_width(1.0),
        );
    }
}

fn draw_axes_labels(
    frame: &mut Frame,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    // Y-axis price labels (right side)
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(plot.x + plot.width + 8.0, y),
            color: TEXT_COLOR,
            size: sp(12.0).into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Left,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }

    // X-axis date labels (bottom). Two rows: only the quarter-start month
    // labels on top, and the quarter labels ("Q1".."Q4") on a second row
    // below. Both rows enforce a minimum pixel gap between label centers so
    // they never overlap, regardless of plot width or how many quarters are
    // shown.
    const MIN_LABEL_SPACING_PX: f32 = 70.0;

    // Top row: quarter-start month labels (e.g. "JAN '26").
    let m_ticks = axis::quarter_month_ticks(x_min, x_max);
    let mut last_label_x: Option<f32> = None;
    for (i, t) in m_ticks.iter().enumerate() {
        let is_last = i == m_ticks.len() - 1;
        let x = data_x_to_screen(t.position, x_min, x_max, plot);
        if x < plot.x || x > plot.x + plot.width {
            continue;
        }
        let spaced = match last_label_x {
            None => true,
            Some(prev_x) => (x - prev_x).abs() >= MIN_LABEL_SPACING_PX,
        };
        if !spaced && !is_last {
            continue;
        }
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(x, plot.y + plot.height + 8.0),
            color: TEXT_COLOR,
            size: sp(12.0).into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });
        last_label_x = Some(x);
    }

    // Bottom row: quarter labels (e.g. "Q1").
    let q_ticks = axis::quarter_ticks(x_min, x_max);
    let mut last_q_x: Option<f32> = None;
    for (i, t) in q_ticks.iter().enumerate() {
        let is_last = i == q_ticks.len() - 1;
        let x = data_x_to_screen(t.position, x_min, x_max, plot);
        if x < plot.x || x > plot.x + plot.width {
            continue;
        }
        let spaced = match last_q_x {
            None => true,
            Some(prev_x) => (x - prev_x).abs() >= MIN_LABEL_SPACING_PX,
        };
        if !spaced && !is_last {
            continue;
        }
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(x, plot.y + plot.height + 24.0),
            color: TEXT_COLOR,
            size: sp(12.0).into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });
        last_q_x = Some(x);
    }
}

/// Return today's candle and its index in the candles array.
fn today_candle(candles: &[Candle]) -> Option<(Candle, usize)> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let today_midnight = now - (now % 86_400);
    let i = candles.iter().position(|c| c.timestamp == today_midnight)?;
    Some((candles[i], i))
}

/// Return the most recent candle and its index, used to keep the default
/// tooltip visible while the new day's candle has not been fetched yet.
fn last_candle(candles: &[Candle]) -> Option<(Candle, usize)> {
    let i = candles.len().checked_sub(1)?;
    Some((candles[i], i))
}

fn draw_crosshair(
    frame: &mut Frame,
    plot: &Rectangle,
    candle: &Candle,
    active_idx: usize,
    all_candles: &[Candle],
    x_min: f64,
    x_max: f64,
    show_line: bool,
    flash: Option<crate::modules::ui::ws_flash::WsFlash>,
) {
    // Vertical line (only shown on hover)
    if show_line {
        let x = data_x_to_screen(candle.timestamp as f64, x_min, x_max, plot);
        if x >= plot.x && x <= plot.x + plot.width {
            let vline = Path::new(|p| {
                p.move_to(Point::new(x, plot.y));
                p.line_to(Point::new(x, plot.y + plot.height));
            });
            frame.stroke(&vline, Stroke::default().with_color(CROSSHAIR_COLOR).with_width(1.0));
        }
    }

    // Price readout — date, close price, and VWAP (top-left corner)
    fn fmt_price_with_commas(p: f64) -> String {
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

    fn fmt_price_whole(p: f64) -> String {
        let whole = p.trunc() as i64;
        let s = whole.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3 + 1);
        result.push('$');
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result
    }

    // Compute the progressive VWAP up to the active index
    let vwap_label = {
        let pairs: Vec<(f64, f64)> = all_candles[..=active_idx]
            .iter()
            .map(|c| (c.close, c.volume))
            .collect();
        let vwaps = crate::modules::compute::vwap::progressive_vwap(&pairs);
        vwaps
            .last()
            .and_then(|v| *v)
            .map(|vwap| format!(" — VWAP: {}", fmt_price_whole(vwap)))
            .unwrap_or_default()
    };

    let head = if let Some(dt) = DateTime::from_timestamp(candle.timestamp, 0) {
        format!(
            "{} {} '{} \u{2014} H: {}  L: {}  C: ",
            dt.day(),
            match dt.month() {
                1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                _ => "???",
            },
            dt.year() % 100,
            fmt_price_with_commas(candle.high),
            fmt_price_with_commas(candle.low),
        )
    } else {
        format!(
            "H: {}  L: {}  C: ",
            fmt_price_with_commas(candle.high),
            fmt_price_with_commas(candle.low),
        )
    };

    let number = crate::modules::ui::ws_flash::format_usd(candle.close);
    let close_value = format!("${}", number);

    // Which byte offset in `number` begins the changed digits, if any.
    let diff = flash.and_then(|f| f.diff_vs(candle.close));
    let flash_color = flash.map(|f| f.color());

    // Draw one tooltip segment at `x` and return the x for the next segment.
    // "Geist Mono" is monospace, so a segment's width is ~0.6 * font size per
    // character; this keeps the colored close adjacent to the white prefix.
    let draw_segment = |frame: &mut Frame, x: f32, y: f32, text: &str, color: Color| -> f32 {
        if text.is_empty() {
            return x;
        }
        frame.fill_text(canvas::Text {
            content: text.to_string(),
            position: Point::new(x, y),
            color,
            size: sp(14.0).into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Left,
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });
        x + text.chars().count() as f32 * sp(14.0) * 0.6
    };

    let y = plot.y + 4.0;
    let mut x = plot.x + 4.0;
    match diff {
        Some(i) if i < number.len() => {
            if let Some(color) = flash_color {
                let white_prefix = format!("{}${}", head, &number[..i]);
                x = draw_segment(frame, x, y, &white_prefix, Color::WHITE);
                x = draw_segment(frame, x, y, &number[i..], color);
                draw_segment(frame, x, y, &vwap_label, Color::WHITE);
            } else {
                let whole = format!("{}{}{}", head, close_value, vwap_label);
                draw_segment(frame, x, y, &whole, Color::WHITE);
            }
        }
        _ => {
            let whole = format!("{}{}{}", head, close_value, vwap_label);
            draw_segment(frame, x, y, &whole, Color::WHITE);
        }
    }
}