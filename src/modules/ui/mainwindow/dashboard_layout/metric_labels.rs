use crate::modules::compute::metrics::Metrics;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use iced::widget::{Row, container, text};
use iced::{Color, Element, Length};
use iced::widget::text::{LineHeight, Wrapping};

/// Line chart colours reused for P/L indicators.
const GREEN_LINE: Color = Color::from_rgb(0.0, 0.8, 0.3);
const RED_LINE: Color = Color::from_rgb(1.0, 0.1, 0.05);
const GREY_LINE: Color = Color::from_rgb(0.5, 0.5, 0.5);

/// Semibold monospaced font used for metric values.
const VALUE_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("Geist Mono"),
    weight: iced::font::Weight::Semibold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

fn metric_card<'a>(
    label: &'a str,
    value: &'a str,
    value_color: Color,
    background: Option<Color>,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let inner = iced::widget::Column::with_children(vec![
        text(label)
            .size(sp(16.0))
            .color(theme::HALVING_BUTTON_TEXT)
            .wrapping(Wrapping::Word)
            .width(Length::Fill)
            .into(),
        text(value)
            .size(sp(19.0))
            .width(Length::Fill)
            .line_height(LineHeight::Relative(1.2))
            .align_x(text::Alignment::Right)
            .font(VALUE_FONT)
            .color(value_color)
            .wrapping(Wrapping::None)
            .into(),
    ])
    .spacing(sp(4.0))
    .padding(iced::Padding::new(sp(8.0)));

    container(inner)
        .width(Length::Fill)
        .style(move |_theme| container::Style {
            background: background.map(iced::Background::Color),
            border: iced::border::rounded(8),
            ..Default::default()
        })
        .into()
}

/// Determine the colour for a P/L value based on its triangle prefix.
fn p_l_color(val: &str) -> Color {
    if val.starts_with('\u{25B2}') {
        GREEN_LINE
    } else if val.starts_with('\u{25BC}') {
        RED_LINE
    } else {
        GREY_LINE
    }
}

pub fn view<'a>(
    metrics: &'a Metrics,
    subsidy: &'a str,
    calmar_click: crate::modules::ui::mainwindow::application::Message,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    // The Calmar card is transparent so the button's own fill shows through,
    // matching the halving sidebar buttons' hover-fill effect.
    let calmar_card = metric_card("Calmar", &metrics.calmar, theme::HALVING_BUTTON_TEXT, None);
    let calmar_button = iced::widget::button(calmar_card)
        .on_press(calmar_click)
        .padding(0)
        .style(|_theme, status| iced::widget::button::Style {
            background: Some(iced::Background::Color(match status {
                iced::widget::button::Status::Hovered => theme::HALVING_BUTTON_HOVER,
                _ => theme::HALVING_BUTTON_BACKGROUND,
            })),
            border: iced::border::rounded(8),
            shadow: Default::default(),
            text_color: Default::default(),
            snap: false,
        });

    Row::with_children(vec![
        metric_card(
            "P/L",
            &metrics.p_l,
            p_l_color(&metrics.p_l),
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        metric_card(
            "High",
            &metrics.high,
            theme::HALVING_BUTTON_TEXT,
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        metric_card(
            "Low",
            &metrics.low,
            theme::HALVING_BUTTON_TEXT,
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        metric_card(
            "Max Draw-Down",
            &metrics.draw_down,
            theme::HALVING_BUTTON_TEXT,
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        metric_card(
            "Max Run-Up",
            &metrics.run_up,
            theme::HALVING_BUTTON_TEXT,
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        metric_card(
            "Subsidy",
            subsidy,
            theme::HALVING_BUTTON_TEXT,
            Some(theme::HALVING_BUTTON_BACKGROUND),
        ),
        calmar_button.into(),
    ])
    .spacing(sp(8.0))
    .width(Length::Fill)
    .into()
}
