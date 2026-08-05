use iced::mouse;
use iced::widget::button;
use iced::widget::canvas::{self, Frame, Path, Stroke, Fill, Style};
use iced::widget::{row, text};
use iced::{Color, Element, Point, Rectangle};
use crate::modules::compute::vwap::progressive_vwap;
use crate::modules::compute::year_over_year::Candle;
use crate::modules::ui::line_chart::state::{DrawingMode, LineChartState, RangeBox};
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;

// ── Tool toggle buttons ─────────────────────────────────────────────────

fn tool_button<'a>(
    label: &'a str,
    active: bool,
    on_press: crate::modules::ui::mainwindow::application::Message,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let bg = if active {
        Color::from_rgba(0.25, 0.5, 1.0, 0.5)
    } else {
        Color::from_rgba(0.4, 0.4, 0.4, 0.3)
    };

    button(
        text(label)
            .size(sp(12.0))
            .color(if active { Color::WHITE } else { theme::HALVING_BUTTON_TEXT }),
    )
    .on_press(on_press)
    .padding(iced::Padding::new(sp(4.0)).horizontal(sp(10.0)))
    .style(move |_theme, _status| button::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::border::rounded(10),
        shadow: Default::default(),
        text_color: Default::default(),
        snap: false,
    })
    .into()
}

pub fn view<'a>(
    active: DrawingMode,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    row![
        tool_button(
            "AVWAP",
            active == DrawingMode::AVWAP,
            crate::modules::ui::mainwindow::application::Message::SelectAVWAP,
        ),
        tool_button(
            "Range",
            active == DrawingMode::Range,
            crate::modules::ui::mainwindow::application::Message::SelectRange,
        ),
    ]
    .spacing(sp(6.0))
    .into()
}

// ── Coordinate helpers (copied from line_chart.rs to avoid circular deps) ──

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

// ── AVWAP drawing ────────────────────────────────────────────────────────

const VWAP_COLOR: Color = Color::WHITE;
/// Tolerance in screen pixels for right-click hit-testing on anchored VWAP lines.
const VWAP_HIT_TOLERANCE: f64 = 5.0;

/// Draw anchored VWAP lines (white, 1.5px) starting from user-selected candle indices.
pub fn draw_anchored_vwaps(
    frame: &mut Frame,
    plot: &Rectangle,
    candles: &[Candle],
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    anchors: &[usize],
) {
    for &anchor_idx in anchors {
        if anchor_idx >= candles.len() {
            continue;
        }

        let sub_candles = &candles[anchor_idx..];
        let pairs: Vec<(f64, f64)> = sub_candles
            .iter()
            .map(|c| (c.close, c.volume))
            .collect();

        let vwaps = progressive_vwap(&pairs);

        let points: Vec<(f64, f64)> = sub_candles
            .iter()
            .zip(vwaps.iter())
            .filter_map(|(c, v)| v.map(|vwap| (c.timestamp as f64, vwap)))
            .collect();

        if points.len() < 2 {
            continue;
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
}

/// Hit-test against all anchored VWAP lines. Returns the list index of the
/// anchor whose line segment is within `VWAP_HIT_TOLERANCE` screen pixels
/// of `(cursor_x, cursor_y)`. Returns `None` if no line is close enough.
pub fn hit_test_anchored_vwaps(
    cursor_x: f64,
    cursor_y: f64,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    candles: &[Candle],
    anchors: &[usize],
) -> Option<usize> {
    for (list_idx, &anchor_idx) in anchors.iter().enumerate() {
        if anchor_idx >= candles.len() {
            continue;
        }

        let sub_candles = &candles[anchor_idx..];
        let pairs: Vec<(f64, f64)> = sub_candles
            .iter()
            .map(|c| (c.close, c.volume))
            .collect();
        let vwaps = progressive_vwap(&pairs);

        let mut prev_pt: Option<(f64, f64)> = None;
        for (c, v) in sub_candles.iter().zip(vwaps.iter()) {
            if let Some(vwap) = v {
                let sx = data_x_to_screen(c.timestamp as f64, x_min, x_max, plot) as f64;
                let sy = data_y_to_screen(*vwap, y_min, y_max, plot) as f64;

                if let Some((px, py)) = prev_pt {
                    let dist = point_to_segment_distance(cursor_x, cursor_y, px, py, sx, sy);
                    if dist <= VWAP_HIT_TOLERANCE {
                        return Some(list_idx);
                    }
                }
                prev_pt = Some((sx, sy));
            } else {
                prev_pt = None;
            }
        }
    }
    None
}

/// Compute the minimum distance from point `p` to the line segment `(a, b)`.
fn point_to_segment_distance(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let abx = bx - ax;
    let aby = by - ay;
    let apx = px - ax;
    let apy = py - ay;

    let t = (apx * abx + apy * aby) / (abx * abx + aby * aby);
    let t = t.clamp(0.0, 1.0);

    let cx = ax + t * abx;
    let cy = ay + t * aby;

    let dx = px - cx;
    let dy = py - cy;
    (dx * dx + dy * dy).sqrt()
}

// ── Range drawing ───────────────────────────────────────────────────────

const RANGE_FILL: Color = Color::from_rgba(0.3, 0.5, 1.0, 0.15);
const RANGE_BORDER: Color = Color::from_rgba(0.3, 0.5, 1.0, 0.6);
const PREVIEW_FILL: Color = Color::from_rgba(0.3, 0.5, 1.0, 0.08);
const PREVIEW_BORDER: Color = Color::from_rgba(0.3, 0.5, 1.0, 0.35);
const RANGE_GREEN: Color = Color::from_rgb(0.0, 0.8, 0.3);
const RANGE_RED: Color = Color::from_rgb(1.0, 0.1, 0.05);

// Dark, slightly transparent pill behind the % label to improve contrast.
const LABEL_PILL_FILL: Color = Color::from_rgba(0.04, 0.05, 0.09, 0.72);
const LABEL_FONT_SIZE: f32 = 15.0;
const LABEL_PADDING_X: f32 = 10.0;
const LABEL_PADDING_Y: f32 = 3.0;
/// Estimated horizontal advance per character for the monospace "Geist Mono" font.
const LABEL_CHAR_WIDTH: f32 = LABEL_FONT_SIZE * 0.6;

/// Draw all completed ranges and any in-progress range preview.
pub fn draw_ranges(
    frame: &mut Frame,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    state: &LineChartState,
) {
    // Completed ranges
    for r in state.ranges.borrow().iter() {
        draw_one_range_box(frame, plot, x_min, x_max, y_min, y_max, r, false);
    }

    // In-progress preview
    if let (Some(from), Some(to)) = (state.range_pending.get(), state.range_preview.get()) {
        let preview = RangeBox {
            from_ts: from.0,
            from_price: from.1,
            to_ts: to.0,
            to_price: to.1,
        };
        draw_one_range_box(frame, plot, x_min, x_max, y_min, y_max, &preview, true);
    }
}

fn draw_one_range_box(
    frame: &mut Frame,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    r: &RangeBox,
    preview: bool,
) {
    let x1 = data_x_to_screen(r.from_ts, x_min, x_max, plot);
    let x2 = data_x_to_screen(r.to_ts, x_min, x_max, plot);
    let y1 = data_y_to_screen(r.from_price, y_min, y_max, plot);
    let y2 = data_y_to_screen(r.to_price, y_min, y_max, plot);

    let left = x1.min(x2);
    let right = x1.max(x2);
    let top = y1.min(y2);
    let bottom = y1.max(y2);

    // Fill
    let fill_color = if preview { PREVIEW_FILL } else { RANGE_FILL };
    let border_color = if preview { PREVIEW_BORDER } else { RANGE_BORDER };

    let rect_path = Path::new(|p| {
        p.move_to(Point::new(left, top));
        p.line_to(Point::new(right, top));
        p.line_to(Point::new(right, bottom));
        p.line_to(Point::new(left, bottom));
        p.close();
    });

    frame.fill(&rect_path, Fill {
        style: Style::Solid(fill_color),
        ..Fill::default()
    });
    frame.stroke(&rect_path, Stroke::default().with_color(border_color).with_width(1.0));

    // % change label. Use the absolute value of the starting price as the
    // base so a negative base cannot flip the sign, and guard against zero.
    let delta = r.to_price - r.from_price;
    let pct = if r.from_price.abs() > f64::EPSILON {
        delta / r.from_price.abs() * 100.0
    } else {
        0.0
    };
    let label = if pct >= 0.0 {
        format!("+{:.2}%", pct)
    } else {
        format!("{:.2}%", pct)
    };
    let label_color = if pct >= 0.0 { RANGE_GREEN } else { RANGE_RED };

    // Position label at the top inside the box
    let label_y = top + 8.0;
    let center_x = (left + right) / 2.0;

    // Pill background behind the label for visibility. The width is sized to
    // the label itself so it is never clipped, even when the range box is
    // narrower than the text.
    let text_width = LABEL_CHAR_WIDTH * label.len() as f32;
    let pill_width = text_width + LABEL_PADDING_X * 2.0;
    let pill_height = LABEL_FONT_SIZE + LABEL_PADDING_Y * 2.0;
    let pill_left = center_x - pill_width / 2.0;
    let pill_center_y = label_y + LABEL_FONT_SIZE / 2.0;
    let pill_top = pill_center_y - pill_height / 2.0;

    let pill_path = Path::new(|p| {
        p.rounded_rectangle(
            Point::new(pill_left, pill_top),
            iced::Size::new(pill_width, pill_height),
            iced::border::radius(pill_height / 2.0),
        );
    });
    frame.fill(&pill_path, Fill {
        style: Style::Solid(LABEL_PILL_FILL),
        ..Fill::default()
    });

    frame.fill_text(canvas::Text {
        content: label,
        position: Point::new(center_x, label_y),
        color: label_color,
        size: LABEL_FONT_SIZE.into(),
        font: iced::Font {
            family: iced::font::Family::Name("Geist Mono"),
            weight: iced::font::Weight::Bold,
            stretch: iced::font::Stretch::Normal,
            style: iced::font::Style::Normal,
        },
        align_x: text::Alignment::Center,
        align_y: iced::alignment::Vertical::Top,
        ..canvas::Text::default()
    });
}

// ── Range event handling ────────────────────────────────────────────────

/// Hit-test cursor against all completed range boxes. Returns the list index
/// of the first hit, or `None`.
fn hit_test_ranges(
    cursor_x: f32,
    cursor_y: f32,
    plot: &Rectangle,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    state: &LineChartState,
) -> Option<usize> {
    for (idx, r) in state.ranges.borrow().iter().enumerate() {
        let x1 = data_x_to_screen(r.from_ts, x_min, x_max, plot);
        let x2 = data_x_to_screen(r.to_ts, x_min, x_max, plot);
        let y1 = data_y_to_screen(r.from_price, y_min, y_max, plot);
        let y2 = data_y_to_screen(r.to_price, y_min, y_max, plot);

        let left = x1.min(x2) - 4.0;
        let right = x1.max(x2) + 4.0;
        let top = y1.min(y2) - 4.0;
        let bottom = y1.max(y2) + 4.0;

        if cursor_x >= left && cursor_x <= right && cursor_y >= top && cursor_y <= bottom {
            return Some(idx);
        }
    }
    None
}

/// Result of processing a range tool mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeActionResult {
    /// No state change.
    None,
    /// State changed — request a redraw (no capture needed).
    Redraw,
    /// State changed — request a redraw and capture further events.
    RedrawAndCapture,
}

/// Handle a mouse event when the Range drawing tool is active.
/// Returns a `RangeActionResult` indicating what the canvas should do.
pub fn handle_range_event(
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: iced::mouse::Cursor,
    state: &LineChartState,
) -> RangeActionResult {
    let plot = Rectangle {
        x: bounds.x + 60.0,
        y: bounds.y + 60.0,
        width: bounds.width - 120.0,
        height: bounds.height - 120.0,
    };
    let cursor_pt = match cursor.position_over(bounds) {
        Some(pt) => pt,
        None => return RangeActionResult::None,
    };

    // Only process if cursor is inside plot area
    if cursor_pt.x < plot.x || cursor_pt.x > plot.x + plot.width
        || cursor_pt.y < plot.y || cursor_pt.y > plot.y + plot.height
    {
        return RangeActionResult::None;
    }

    let (x_min, x_max) = state.x_bounds();
    let (y_min, y_max) = state.y_bounds();
    let data_ts = screen_x_to_data(cursor_pt.x, x_min, x_max, &plot);
    let data_price = screen_y_to_data(cursor_pt.y, y_min, y_max, &plot);

    match event {
        canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if state.range_pending.get().is_none() {
                // First click: set the 'from' point
                state.range_pending.set(Some((data_ts, data_price)));
                state.range_preview.set(Some((data_ts, data_price)));
            } else {
                // Second click: finalize the range
                if let Some(from) = state.range_pending.get() {
                    let r = RangeBox {
                        from_ts: from.0,
                        from_price: from.1,
                        to_ts: data_ts,
                        to_price: data_price,
                    };
                    state.ranges.borrow_mut().push(r);
                }
                state.range_pending.set(None);
                state.range_preview.set(None);
            }
            RangeActionResult::RedrawAndCapture
        }
        canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
            // Update live preview if we have a 'from' point
            if state.range_pending.get().is_some() {
                state.range_preview.set(Some((data_ts, data_price)));
                RangeActionResult::Redraw
            } else {
                RangeActionResult::None
            }
        }
        canvas::Event::Mouse(mouse::Event::CursorLeft) => {
            state.range_preview.set(None);
            RangeActionResult::Redraw
        }
        _ => RangeActionResult::None,
    }
}

/// Handle a global right-click: delete whichever drawing is under the cursor —
/// either a range box or an anchored VWAP line — regardless of the active tool.
/// If nothing is under the cursor, cancels any in-progress range placement.
pub fn handle_right_click_delete(
    cursor: iced::mouse::Cursor,
    bounds: Rectangle,
    state: &LineChartState,
) -> RangeActionResult {
    let plot = Rectangle {
        x: bounds.x + 60.0,
        y: bounds.y + 60.0,
        width: bounds.width - 120.0,
        height: bounds.height - 120.0,
    };
    let cursor_pt = match cursor.position_over(bounds) {
        Some(pt) => pt,
        None => return RangeActionResult::None,
    };

    // Only process if the cursor is inside the plot area.
    if cursor_pt.x < plot.x || cursor_pt.x > plot.x + plot.width
        || cursor_pt.y < plot.y || cursor_pt.y > plot.y + plot.height
    {
        return RangeActionResult::None;
    }

    let (x_min, x_max) = state.x_bounds();
    let (y_min, y_max) = state.y_bounds();

    // Range boxes are drawn on top of the VWAP lines, so hit-test them first.
    if let Some(idx) = hit_test_ranges(
        cursor_pt.x, cursor_pt.y, &plot,
        x_min, x_max, y_min, y_max, state,
    ) {
        state.ranges.borrow_mut().remove(idx);
        state.range_pending.set(None);
        state.range_preview.set(None);
        return RangeActionResult::RedrawAndCapture;
    }

    // Otherwise hit-test the anchored VWAP lines.
    if let Some(list_idx) = hit_test_anchored_vwaps(
        cursor_pt.x as f64,
        cursor_pt.y as f64,
        &plot,
        x_min, x_max,
        y_min, y_max,
        &state.candles,
        &state.anchors(),
    ) {
        state.remove_anchor_at(list_idx);
        state.range_pending.set(None);
        state.range_preview.set(None);
        return RangeActionResult::RedrawAndCapture;
    }

    // Nothing was hit: cancel any in-progress range placement.
    if state.range_pending.get().is_some() {
        state.range_pending.set(None);
        state.range_preview.set(None);
        return RangeActionResult::Redraw;
    }
    RangeActionResult::None
}