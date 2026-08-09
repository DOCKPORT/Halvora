use crate::modules::compute::metrics::PLSign;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use iced::widget::{Column, Row, button, container, scrollable, stack, svg};
use iced::{Color, ContentFit, Element, Length, border};

/// The sidebar banner SVG, embedded in the binary at compile time so the
/// running program does not depend on any path on disk.
const LOGO_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/Halvora_Logo/Halvora.svg"
));

/// Background color pairs for a button based on its P/L sign.
///
/// Positive and negative use a light translucent tint. No-change (including
/// future halvings without data yet) keeps the original grey button style.
fn fill_colors(sign: PLSign) -> (Color, Color) {
    // (normal, hover)
    match sign {
        PLSign::Positive => (theme::BUTTON_FILL_GREEN, theme::BUTTON_FILL_GREEN_HOVER),
        PLSign::Negative => (theme::BUTTON_FILL_RED, theme::BUTTON_FILL_RED_HOVER),
        PLSign::NoChange => (
            theme::HALVING_BUTTON_BACKGROUND,
            theme::HALVING_BUTTON_HOVER,
        ),
    }
}

/// Shared style for the YoY and halving buttons: P/L-tinted fill, hover
/// highlight, and a thicker border when selected.
fn button_style(sign: PLSign, is_selected: bool, status: button::Status) -> button::Style {
    let (fill, fill_hover) = fill_colors(sign);

    let background = match status {
        button::Status::Hovered => fill_hover,
        _ => fill,
    };

    let text_color = theme::HALVING_BUTTON_TEXT;

    let border = if is_selected {
        border::rounded(8)
            .color(theme::HALVING_BUTTON_TEXT)
            .width(1.5)
    } else {
        border::rounded(8)
            .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
            .width(1.0)
    };

    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border,
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

pub fn view<'a>(
    selected_halving: Option<u32>,
    yoy_selected: bool,
    yoy_pl_sign: PLSign,
    halving_pl_signs: &[PLSign],
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;

    // Build 32 buttons in a 2-column grid: 16 rows of [H-n, H-(n+1)]
    let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(16);
    for i in (1..=32).step_by(2) {
        let row = Row::with_children(vec![
            halving_button(
                i,
                selected_halving,
                halving_pl_signs
                    .get(i as usize)
                    .copied()
                    .unwrap_or(PLSign::NoChange),
            ),
            if i < 32 {
                halving_button(
                    i + 1,
                    selected_halving,
                    halving_pl_signs
                        .get((i + 1) as usize)
                        .copied()
                        .unwrap_or(PLSign::NoChange),
                )
            } else {
                container(iced::widget::column![])
                    .width(Length::Fixed(sp(100.0)))
                    .height(Length::Fixed(sp(36.0)))
                    .into()
            },
        ])
        .spacing(sp(8.0))
        .padding(iced::Padding::new(0.0).left(sp(21.0)).right(sp(21.0)))
        .width(Length::Fill)
        .into();
        rows.push(row);
    }

    let content = Column::with_children({
        let mut children: Vec<Element<'a, Message>> = Vec::with_capacity(17);

        // Logo at top — clicking it opens the About dialog. A faint highlight
        // appears on hover to signal that it is clickable.
        children.push(
            button(
                svg::Svg::new(svg::Handle::from_memory(LOGO_SVG))
                    .content_fit(ContentFit::Contain)
                    .width(Length::Fill)
                    .height(Length::Fixed(sp(80.0))),
            )
            .width(Length::Fill)
            .height(Length::Fixed(sp(80.0)))
            .padding(0)
            .on_press(Message::AboutClicked)
            .style(|_theme, status| button::Style {
                background: match status {
                    button::Status::Hovered => Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.05,
                    ))),
                    _ => None,
                },
                border: match status {
                    button::Status::Hovered => iced::border::Border::default()
                        .color(Color::from_rgba(0.85, 0.85, 0.85, 0.2))
                        .width(1.0),
                    _ => iced::border::Border::default(),
                },
                shadow: Default::default(),
                text_color: Default::default(),
                snap: false,
            })
            .into(),
        );

        // Spacer
        children.push(iced::widget::space().height(Length::Fixed(sp(8.0))).into());

        // YoY button — same padding & width as grid rows for centering
        children.push(
            Row::with_children(vec![yoy_button(yoy_selected, yoy_pl_sign)])
                .padding(iced::Padding::new(0.0).left(sp(21.0)).right(sp(21.0)))
                .width(Length::Fill)
                .into(),
        );

        // Spacer before grid
        children.push(iced::widget::space().height(Length::Fixed(sp(8.0))).into());

        // Grid rows
        for row in rows {
            children.push(row);
        }

        children
    })
    .spacing(sp(8.0))
    .padding(0);

    // Cross-hatch lines sit just below the buttons. `Stack` places all
    // children on top of each other but aligns them to the top-left by
    // default, so wrap the scrollable in a full-size transparent container.
    let scrollable_layer: Element<'a, Message> = container(
        scrollable(content)
            .direction(crate::modules::ui::theme::sidebar_scrollbar_direction())
            .style(crate::modules::ui::theme::sidebar_scrollable_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(0)
    .into();

    container(
        stack![
            crate::modules::ui::splash_screen::crosshatch_background::view_with_h_v_padding(
                0.35, 12.0, 0.0
            ),
            scrollable_layer,
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fixed(sp(250.0)))
    .height(Length::Fill)
    .padding(0)
    .style(|_theme| {
        container::Style::default().background(iced::Background::Color(theme::SIDEBAR_BACKGROUND))
    })
    .into()
}

fn yoy_button<'a>(
    is_selected: bool,
    sign: PLSign,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;

    button(
        iced::widget::text("Year-Over-Year")
            .size(sp(16.0))
            .width(Length::Shrink)
            .center()
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            }),
    )
    .width(Length::Fill)
    .height(Length::Fixed(sp(36.0)))
    .padding(0)
    .on_press(Message::YoYSelected)
    .style(move |_theme, status| button_style(sign, is_selected, status))
    .into()
}

fn halving_button<'a>(
    num: u32,
    selected_halving: Option<u32>,
    sign: PLSign,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;
    let is_selected = selected_halving == Some(num);

    button(
        iced::widget::text(format!("H-{num}"))
            .size(sp(16.0))
            .width(Length::Shrink)
            .center()
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            }),
    )
    .width(Length::Fixed(sp(100.0)))
    .height(Length::Fixed(sp(36.0)))
    .padding(0)
    .on_press(Message::HalvingSelected(num))
    .style(move |_theme, status| button_style(sign, is_selected, status))
    .into()
}
