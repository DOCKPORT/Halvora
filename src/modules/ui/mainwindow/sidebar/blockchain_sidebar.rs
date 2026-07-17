use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Length};
use crate::modules::ui::theme;

pub fn view<'a>() -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let content = Column::with_children(vec![
        text("Blockchain Data").size(18).into(),
        text("").size(8).into(),
        text("Block Height: 0").size(14).into(),
        text("Hashrate: --").size(14).into(),
        text("Difficulty: --").size(14).into(),
        text("Mempool: --").size(14).into(),
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