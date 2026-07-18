use iced::widget::{column, container};
use iced::{border, Element, Length};
use crate::modules::ui::theme;

pub fn view<'a>() -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let placeholder_style = |_theme: &iced::Theme| -> container::Style {
        container::Style::default()
            .background(iced::Background::Color(theme::SIDEBAR_BACKGROUND))
            .border(border::rounded(8).color(theme::DASHBOARD_PLACEHOLDER_BORDER).width(2))
    };

    let metrics = container(iced::widget::column![])
        .width(Length::Fill)
        .height(Length::FillPortion(1))
        .style(placeholder_style);

    let price = container(iced::widget::column![])
        .width(Length::Fill)
        .height(Length::FillPortion(7))
        .style(placeholder_style);

    let volume = container(iced::widget::column![])
        .width(Length::Fill)
        .height(Length::FillPortion(2))
        .style(placeholder_style);

    container(
        column![metrics, price, volume]
            .spacing(16)
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| {
        container::Style::default()
            .background(iced::Background::Color(theme::MAINWINDOW_BACKGROUND))
    })
    .into()
}
