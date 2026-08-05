use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke,
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

use crate::modules::ui::scaling;

/// Spacing between adjacent 45° lines, in unscaled reference pixels.
const LINE_SPACING: f32 = 12.0;

/// Width of each 45° line, in unscaled reference pixels.
const LINE_WIDTH: f32 = 2.0;

/// Uniform distance from each edge of the widget where lines stop.
const EDGE_PADDING: f32 = 40.0;

/// The splash background canvas program.
///
/// Draws a single family of parallel lines at 45°, in a subtle grey, over
/// the full widget area. The lines stop a uniform distance (`EDGE_PADDING`)
/// from each edge and fade with the splash's opacity.
pub struct BackgroundProgram {
    /// The splash fade opacity in the range 0.0..=1.0.
    pub opacity: f32,
}

/// Builds the splash background canvas widget.
pub fn view(opacity: f32) -> Element<'static, crate::modules::ui::mainwindow::application::Message> {
    Canvas::new(BackgroundProgram { opacity })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

impl<Message> canvas::Program<Message> for BackgroundProgram {
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

        let spacing = scaling::sp(LINE_SPACING);
        let line_width = scaling::sp(LINE_WIDTH);
        let padding = scaling::sp(EDGE_PADDING);

        let left = padding;
        let top = padding;
        let right = bounds.width - padding;
        let bottom = bounds.height - padding;

        if right <= left || bottom <= top {
            return vec![frame.into_geometry()];
        }

        // Line colour tuned to the theme's grey, kept subtle.
        let line_color = Color::from_rgba(0.6, 0.6, 0.6, 0.25).scale_alpha(self.opacity);
        let stroke = Stroke::default()
            .with_color(line_color)
            .with_width(line_width);

        // Lines slope such that as x increases, y also increases (down-right).
        // For a line with slope 1 starting at screen y, a point offset dy
        // below it has its starting x reduced by the same dy.
        //
        // Walk each possible starting row at the left edge (top to bottom),
        // then walk the remaining starting columns at the top edge (left to
        // right). Each starting point produces one 45° segment clipped to the
        // padded rectangle.
        let mut start_y = top;
        while start_y <= bottom {
            draw_segment(&mut frame, stroke, left, start_y, right, bottom);
            start_y += spacing;
        }

        let mut start_x = left + spacing;
        while start_x <= right {
            draw_segment(&mut frame, stroke, start_x, top, right, bottom);
            start_x += spacing;
        }

        vec![frame.into_geometry()]
    }
}

/// Draws one 45° line segment from its start point, clipped to the padded
/// rectangle's right and bottom edges.
fn draw_segment(frame: &mut Frame, stroke: Stroke, start_x: f32, start_y: f32, right: f32, bottom: f32) {
    let horiz_dist = right - start_x;
    let vert_dist = bottom - start_y;
    let dist = horiz_dist.min(vert_dist);

    let end_x = start_x + dist;
    let end_y = start_y + dist;

    let path = Path::new(|builder| {
        builder.move_to(Point::new(start_x, start_y));
        builder.line_to(Point::new(end_x, end_y));
    });
    frame.stroke(&path, stroke);
}
