use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Length};
use crate::modules::ui::theme;

fn info_card<'a>(title: &'a str, value: String) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let inner = Column::with_children(vec![
        text(title)
            .size(14)
            .color(theme::HALVING_BUTTON_TEXT)
            .into(),
        text(value)
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

pub fn view<'a>(current_tip_height: u32, current_subsidy_sat: i64, next_halving_eta: &str, coins_issued: &str, percentage_issued: &str, remaining_issuance: &str) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let height_str = current_tip_height.to_string();
    let subsidy_btc = current_subsidy_sat as f64 / 100_000_000.0;
    let subsidy_str = if current_subsidy_sat == 0 {
        "xxxxx".to_string()
    } else {
        format!("{:.8}", subsidy_btc)
    };
    let content = Column::with_children(vec![
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Block Height", height_str),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Current Subsidy", subsidy_str),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Next Halving", next_halving_eta.to_string()),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Coins Minted", coins_issued.to_string()),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Percentage Issued", percentage_issued.to_string()),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Remaining Issuance", remaining_issuance.to_string()),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("Sats per USD", "xxxxx".to_string()),
        iced::widget::space().height(Length::Fixed(8.0)).into(),
        info_card("All-Time High", "xxxxx".to_string()),
    ])
    .spacing(0)
    .padding(iced::Padding::new(0.0).left(21.0).right(21.0));

    container(scrollable(content))
        .width(Length::Fixed(250.0))
        .height(Length::Fill)
        .padding(0)
        .style(|_theme| {
            container::Style::default().background(
                iced::Background::Color(theme::SIDEBAR_BACKGROUND)
            )
        })
        .into()
}