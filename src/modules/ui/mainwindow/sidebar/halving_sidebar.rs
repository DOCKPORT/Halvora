use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Length};
use crate::modules::ui::theme;

pub fn view<'a>() -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let content = Column::with_children(vec![
        text("Halvings").size(18).into(),
        text("").size(8).into(),
        text("1 - Block 0").size(14).into(),
        text("2 - Block 210,000").size(14).into(),
        text("3 - Block 420,000").size(14).into(),
        text("4 - Block 630,000").size(14).into(),
        text("5 - Block 840,000").size(14).into(),
    ]);

    container(scrollable(content))
        .width(Length::Fixed(250.0))
        .height(Length::Fill)
        .style(|_theme| {
            container::Style::default().background(
                iced::Background::Color(theme::SIDEBAR_BACKGROUND)
            )
        })
        .into()
}