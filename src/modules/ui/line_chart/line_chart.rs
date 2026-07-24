use chrono::{DateTime, Datelike};
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

/// Crosshair state tracked internally by the canvas program on mouse move.
#[derive(Default, Clone, Copy)]
struct CrosshairState {
    ts: Option<f64>,
    price: Option<f64>,
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

        // 2. Price line with fill
        draw_price_line(
            &mut frame,
            &plot,
            &self.data.candles,
            x_min,
            x_max,
            y_min,
            y_max,
        );

        // 3. Progressive VWAP line (white, on top of price line)
        draw_vwap_line(
            &mut frame,
            &plot,
            &self.data.candles,
            x_min,
            x_max,
            y_min,
            y_max,
        );

        // 4. Axes labels
        draw_axes_labels(&mut frame, &plot, x_min, x_max, y_min, y_max);

        // 4. Crosshair
        if let (Some(ts), Some(price)) = (state.ts, state.price) {
            draw_crosshair(&mut frame, &plot, ts, price, x_min, x_max, y_min, y_max);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut CrosshairState,
        event: &canvas::Event,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { position } => {
                    // In update(), bounds are screen-absolute and cursor
                    // is screen-absolute, so we must keep bounds.x/y here.
                    let plot = Rectangle {
                        x: bounds.x + 60.0,
                        y: bounds.y + 60.0,
                        width: bounds.width - 120.0,
                        height: bounds.height - 120.0,
                    };
                    if position.x >= plot.x
                        && position.x <= plot.x + plot.width
                        && position.y >= plot.y
                        && position.y <= plot.y + plot.height
                    {
                        let (x_min, x_max) = self.data.x_bounds();
                        let (y_min, y_max) = self.data.y_bounds();
                        state.ts = Some(screen_x_to_data(position.x, x_min, x_max, &plot));
                        state.price = Some(screen_y_to_data(position.y, y_min, y_max, &plot));
                    } else {
                        state.ts = None;
                        state.price = None;
                    }
                    Some(canvas::Action::request_redraw().and_capture())
                }
                mouse::Event::CursorLeft => {
                    state.ts = None;
                    state.price = None;
                    Some(canvas::Action::request_redraw().and_capture())
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

fn draw_axes_labels(
    frame: &mut Frame,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    // Y-axis price labels (left side)
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(plot.x - 8.0, y),
            color: TEXT_COLOR,
            size: 12.0.into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Right,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }

    // X-axis date labels (bottom)
    let ticks = axis::x_ticks(x_min, x_max);
    let max_labels = 8;
    let step = if ticks.len() > max_labels {
        ticks.len() / max_labels
    } else {
        1
    };
    for (i, t) in ticks.iter().enumerate() {
        if i % step != 0 && i != ticks.len() - 1 {
            continue;
        }
        let x = data_x_to_screen(t.position, x_min, x_max, plot);
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(x, plot.y + plot.height + 8.0),
            color: TEXT_COLOR,
            size: 12.0.into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });
    }
}

fn draw_crosshair(
    frame: &mut Frame,
    plot: &Rectangle,
    ts: f64,
    price: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) {
    let x = data_x_to_screen(ts, x_min, x_max, plot);
    let y = data_y_to_screen(price, y_min, y_max, plot);

    // Vertical line
    if x >= plot.x && x <= plot.x + plot.width {
        let vline = Path::new(|p| {
            p.move_to(Point::new(x, plot.y));
            p.line_to(Point::new(x, plot.y + plot.height));
        });
        frame.stroke(&vline, Stroke::default().with_color(CROSSHAIR_COLOR).with_width(1.0));
    }

    // Horizontal line
    if y >= plot.y && y <= plot.y + plot.height {
        let hline = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(&hline, Stroke::default().with_color(CROSSHAIR_COLOR).with_width(1.0));
    }

    // Price readout — date and price side by side (top-right corner)
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

    let label = if let Some(dt) = DateTime::from_timestamp(ts as i64, 0) {
        format!(
            "{} {} '{} \u{2014}  {}",
            dt.day(),
            match dt.month() {
                1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                _ => "???",
            },
            dt.year() % 100,
            fmt_price_with_commas(price),
        )
    } else {
        fmt_price_with_commas(price)
    };

    frame.fill_text(canvas::Text {
        content: label,
        position: Point::new(plot.x + plot.width - 4.0, plot.y + 4.0),
        color: Color::WHITE,
        size: 14.0.into(),
        font: iced::Font::with_name("Geist Mono"),
        align_x: text::Alignment::Right,
        align_y: iced::alignment::Vertical::Top,
        ..canvas::Text::default()
    });
}