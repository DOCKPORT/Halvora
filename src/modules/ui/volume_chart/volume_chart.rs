use iced::mouse;
use iced::widget::canvas::{self, Canvas, Fill, Frame, Geometry, Path, Stroke, Style};
use iced::widget::text;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use super::axis;
use super::state;
use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::line_chart::LineChartState;
use crate::modules::ui::scaling::sp;

/// The volume bar chart widget.
pub struct VolumeChart<'a> {
    chart_state: &'a LineChartState,
    width: Length,
    height: Length,
}

impl<'a> VolumeChart<'a> {
    pub fn new(chart_state: &'a LineChartState) -> Self {
        Self {
            chart_state,
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

impl<'a, Message> From<VolumeChart<'a>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(chart: VolumeChart<'a>) -> Element<'a, Message> {
        Canvas::new(VolumeChartProgram {
            chart_state: chart.chart_state,
        })
        .width(chart.width)
        .height(chart.height)
        .into()
    }
}

struct VolumeChartProgram<'a> {
    chart_state: &'a LineChartState,
}

impl<Message> canvas::Program<Message> for VolumeChartProgram<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let candles = &self.chart_state.candles;
        if candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let (x_min, x_max) = state::x_bounds(candles);
        let (y_min, y_max) = state::y_bounds(candles);
        let plot = padded_plot_area(bounds);

        // 1. Grid lines at 33% and 67%
        draw_grid(&mut frame, &plot, y_min, y_max);

        // 2. Grey border around the plot area
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

        // 3. Volume bars
        draw_bars(&mut frame, &plot, candles, x_min, x_max, y_min, y_max);

        // 4. Quarter vertical lines (aligned with line chart)
        let quarter_color = Color::from_rgba(0.6, 0.6, 0.6, 0.3);
        for q in crate::modules::ui::line_chart::axis::quarter_ticks(x_min, x_max) {
            let x = data_x_to_screen(q.position, x_min, x_max, &plot);
            if x < plot.x || x > plot.x + plot.width {
                continue;
            }
            let qline_path = vertical_line(&plot, x);
            frame.stroke(
                &qline_path,
                Stroke::default().with_color(quarter_color).with_width(1.0),
            );
        }

        // 5. Y-axis labels
        draw_axis_labels(&mut frame, &plot, y_min, y_max);

        // 6. Crosshair vertical line synced from the line chart
        if let Some(idx) = self.chart_state.hovered_index.get()
            && idx < candles.len()
        {
            let x = data_x_to_screen(candles[idx].timestamp as f64, x_min, x_max, &plot);
            if x >= plot.x && x <= plot.x + plot.width {
                let vline = vertical_line(&plot, x);
                frame.stroke(
                    &vline,
                    Stroke::default()
                        .with_color(CROSSHAIR_COLOR)
                        .with_width(1.0),
                );
            }
        }

        // 7. Volume tooltip in top-left corner
        let tooltip_idx = self
            .chart_state
            .hovered_index
            .get()
            .or_else(|| today_candle(candles).map(|(_, idx)| idx));
        if let Some(idx) = tooltip_idx
            && idx < candles.len()
        {
            let vol = candles[idx].volume;
            frame.fill_text(canvas::Text {
                content: format!("VOL: {vol:.2}"),
                position: Point::new(plot.x + 4.0, plot.y + 4.0),
                color: Color::WHITE,
                size: sp(14.0).into(),
                font: iced::Font::with_name("Geist Mono"),
                align_x: text::Alignment::Left,
                align_y: iced::alignment::Vertical::Top,
                ..canvas::Text::default()
            });
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut (),
        _event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        None
    }
}

// ── Layout ───────────────────────────────────────────────────────────────

/// Plot area with padding. Left 60px for labels, bottom 40px for breathing room.
fn padded_plot_area(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: 60.0,
        y: 25.0,
        width: bounds.width - 120.0,
        height: bounds.height - 50.0,
    }
}

// ── Coordinate conversion ────────────────────────────────────────────────

fn data_x_to_screen(ts: f64, x_min: f64, x_max: f64, plot: &Rectangle) -> f32 {
    let t = (ts - x_min) / (x_max - x_min);
    plot.x + (t as f32) * plot.width
}

fn data_y_to_screen(volume: f64, y_min: f64, y_max: f64, plot: &Rectangle) -> f32 {
    let t = (volume - y_min) / (y_max - y_min);
    plot.y + (1.0 - t as f32) * plot.height
}

/// A full-height vertical line from the top to the bottom of the plot area.
fn vertical_line(plot: &Rectangle, x: f32) -> Path {
    Path::new(|p| {
        p.move_to(Point::new(x, plot.y));
        p.line_to(Point::new(x, plot.y + plot.height));
    })
}

// ── Colour constants ─────────────────────────────────────────────────────

const GRID_COLOR: Color = Color::from_rgba(0.3, 0.3, 0.3, 0.45);
const TEXT_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7);
const BAR_HIGH: Color = Color::from_rgb(1.0, 1.0, 1.0);
const BAR_MID: Color = Color::from_rgb(0.7, 0.7, 0.7);
const BAR_LOW: Color = Color::from_rgb(0.35, 0.35, 0.35);
const CROSSHAIR_COLOR: Color = Color::from_rgba(0.8, 0.8, 0.8, 0.5);

// ── Drawing helpers ──────────────────────────────────────────────────────

fn draw_grid(frame: &mut Frame, plot: &Rectangle, y_min: f64, y_max: f64) {
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        let path = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(
            &path,
            Stroke::default().with_color(GRID_COLOR).with_width(0.5),
        );
    }
}

fn draw_bars(
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

    let range = y_max - y_min;
    let threshold_66 = range * 0.6667;
    let threshold_33 = range * 0.3333;
    let plot_bottom = plot.y + plot.height;
    let bar_width = plot.width / candles.len() as f32 * 0.7;

    for c in candles {
        let x = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot);

        let bar_top = data_y_to_screen(c.volume, y_min, y_max, plot);
        let bar_bottom = plot_bottom;

        if bar_bottom - bar_top <= 0.0 {
            continue;
        }

        let vol_y = c.volume - y_min;
        let bar_color = if vol_y >= threshold_66 {
            BAR_HIGH
        } else if vol_y >= threshold_33 {
            BAR_MID
        } else {
            BAR_LOW
        };

        let rect_path = Path::new(|p| {
            p.move_to(Point::new(x - bar_width / 2.0, bar_top));
            p.line_to(Point::new(x + bar_width / 2.0, bar_top));
            p.line_to(Point::new(x + bar_width / 2.0, bar_bottom));
            p.line_to(Point::new(x - bar_width / 2.0, bar_bottom));
            p.close();
        });
        frame.fill(
            &rect_path,
            Fill {
                style: Style::Solid(bar_color),
                ..Fill::default()
            },
        );
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

fn draw_axis_labels(frame: &mut Frame, plot: &Rectangle, y_min: f64, y_max: f64) {
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        frame.fill_text(canvas::Text {
            content: t.label.clone(),
            position: Point::new(plot.x - 8.0, y),
            color: TEXT_COLOR,
            size: sp(12.0).into(),
            font: iced::Font::with_name("Geist Mono"),
            align_x: text::Alignment::Right,
            align_y: iced::alignment::Vertical::Center,
            ..canvas::Text::default()
        });
    }
}
