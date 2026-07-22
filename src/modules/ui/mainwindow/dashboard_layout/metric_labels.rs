use iced::widget::{container, text, Row};
use iced::{Element, Length};
use crate::modules::ui::theme;

fn metric_card<'a>(
    label: &'a str,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let inner = iced::widget::Column::with_children(vec![
        text(label)
            .size(14)
            .color(theme::HALVING_BUTTON_TEXT)
            .into(),
        text("\u{2014}")
            .size(16)
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            })
            .color(theme::HALVING_BUTTON_TEXT)
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

pub fn view<'a>() -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    Row::with_children(vec![
        metric_card("P/L"),
        metric_card("High"),
        metric_card("Low"),
        metric_card("Draw-Down"),
        metric_card("Run-Up"),
        metric_card("Calmar"),
    ])
    .spacing(8)
    .width(Length::Fill)
    .into()
}