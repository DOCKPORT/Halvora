use iced::widget::{container, text, Row};
use iced::{Color, Element, Length};
use crate::modules::compute::metrics::Metrics;
use crate::modules::ui::theme;

/// Line chart colours reused for P/L indicators.
const GREEN_LINE: Color = Color::from_rgb(0.0, 0.8, 0.3);
const RED_LINE: Color = Color::from_rgb(1.0, 0.1, 0.05);
const GREY_LINE: Color = Color::from_rgb(0.5, 0.5, 0.5);

fn metric_card<'a>(
    label: &'a str,
    value: &'a str,
    value_color: Color,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let inner = iced::widget::Column::with_children(vec![
        text(label)
            .size(15)
            .color(theme::HALVING_BUTTON_TEXT)
            .into(),
        text(value)
            .size(18)
            .width(Length::Fill)
            .align_x(text::Alignment::Right)
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            })
            .color(value_color)
            .into(),
    ])
    .spacing(4)
    .padding(iced::Padding::new(8.0));

    container(inner)
        .width(Length::Fill)
        .style(|_theme| {
            container::Style {
                background: Some(iced::Background::Color(theme::HALVING_BUTTON_BACKGROUND)),
                border: iced::border::rounded(8),
                ..Default::default()
            }
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
    let calmar_card = metric_card("Calmar", &metrics.calmar, theme::HALVING_BUTTON_TEXT);
    let calmar_button = iced::widget::button(calmar_card)
        .on_press(calmar_click)
        .padding(0)
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            border: iced::border::rounded(0),
            shadow: Default::default(),
            text_color: Default::default(),
            snap: false,
        });

    Row::with_children(vec![
        metric_card("P/L", &metrics.p_l, p_l_color(&metrics.p_l)),
        metric_card("High", &metrics.high, theme::HALVING_BUTTON_TEXT),
        metric_card("Low", &metrics.low, theme::HALVING_BUTTON_TEXT),
        metric_card("Max Draw-Down", &metrics.draw_down, theme::HALVING_BUTTON_TEXT),
        metric_card("Max Run-Up", &metrics.run_up, theme::HALVING_BUTTON_TEXT),
        metric_card("Subsidy", subsidy, theme::HALVING_BUTTON_TEXT),
        calmar_button.into(),
    ])
    .spacing(8)
    .width(Length::Fill)
    .into()
}
