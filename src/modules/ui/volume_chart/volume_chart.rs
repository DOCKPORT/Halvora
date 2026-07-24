use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Fill, Style,
};
use iced::widget::text;
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use super::axis;
use super::state;
use crate::modules::compute::year_over_year::Candle;

/// The volume bar chart widget.
pub struct VolumeChart<'a> {
    candles: &'a [Candle],
    width: Length,
    height: Length,
}

impl<'a> VolumeChart<'a> {
    pub fn new(candles: &'a [Candle]) -> Self {
        Self {
            candles,
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
            candles: chart.candles,
        })
        .width(chart.width)
        .height(chart.height)
        .into()
    }
}

struct VolumeChartProgram<'a> {
    candles: &'a [Candle],
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

        if self.candles.is_empty() {
            return vec![frame.into_geometry()];
        }

        let (x_min, x_max) = state::x_bounds(self.candles);
        let (y_min, y_max) = state::y_bounds(self.candles);
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
        draw_bars(&mut frame, &plot, self.candles, x_min, x_max, y_min, y_max);

        // 4. Y-axis labels
        draw_axis_labels(&mut frame, &plot, y_min, y_max);

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

// ── Colour constants ─────────────────────────────────────────────────────

const GRID_COLOR: Color = Color::from_rgba(0.3, 0.3, 0.3, 0.45);
const TEXT_COLOR: Color = Color::from_rgb(0.7, 0.7, 0.7);
const BAR_HIGH: Color = Color::from_rgb(1.0, 1.0, 1.0);
const BAR_MID: Color = Color::from_rgb(0.7, 0.7, 0.7);
const BAR_LOW: Color = Color::from_rgb(0.35, 0.35, 0.35);

// ── Drawing helpers ──────────────────────────────────────────────────────

fn draw_grid(frame: &mut Frame, plot: &Rectangle, y_min: f64, y_max: f64) {
    let ticks = axis::y_ticks(y_min, y_max);
    for t in &ticks {
        let y = data_y_to_screen(t.position, y_min, y_max, plot);
        let path = Path::new(|p| {
            p.move_to(Point::new(plot.x, y));
            p.line_to(Point::new(plot.x + plot.width, y));
        });
        frame.stroke(&path, Stroke::default().with_color(GRID_COLOR).with_width(0.5));
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

fn draw_axis_labels(frame: &mut Frame, plot: &Rectangle, y_min: f64, y_max: f64) {
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
}