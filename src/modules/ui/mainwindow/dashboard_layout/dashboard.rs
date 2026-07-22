use iced::widget::{column, container, row, text};
use iced::{border, Element, Length};
use crate::modules::ui::theme;
use super::metric_labels;

fn ordinal_suffix(n: u32) -> &'static str {
    match n % 100 {
        11 | 12 | 13 => "TH",
        _ => match n % 10 {
            1 => "ST",
            2 => "ND",
            3 => "RD",
            _ => "TH",
        },
    }
}

pub fn view<'a>(selected_halving: Option<u32>, yoy_selected: bool) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let placeholder_style = |_theme: &iced::Theme| -> container::Style {
        container::Style::default()
            .background(iced::Background::Color(theme::SIDEBAR_BACKGROUND))
            .border(border::rounded(8).color(theme::DASHBOARD_PLACEHOLDER_BORDER).width(1.5))
    };

    let metrics_label: iced::widget::Column<'_, crate::modules::ui::mainwindow::application::Message> = if yoy_selected {
        column![
            text("Year")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Shrink),
            text("Over")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Shrink),
            text("Year")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Shrink),
        ]
        .width(Length::Shrink)
    } else {
        selected_halving.map_or(
            iced::widget::column![].width(Length::Shrink),
            |n| {
                column![
                    text(format!("{}{}", n, ordinal_suffix(n)))
                        .size(18)
                        .font(iced::Font {
                            family: iced::font::Family::Name("Geist Mono"),
                            weight: iced::font::Weight::Normal,
                            stretch: iced::font::Stretch::Normal,
                            style: iced::font::Style::Normal,
                        })
                        .color(theme::HALVING_BUTTON_TEXT)
                        .width(Length::Shrink),
                    text("HALVING")
                        .size(18)
                        .font(iced::Font {
                            family: iced::font::Family::Name("Geist Mono"),
                            weight: iced::font::Weight::Normal,
                            stretch: iced::font::Stretch::Normal,
                            style: iced::font::Style::Normal,
                        })
                        .color(theme::HALVING_BUTTON_TEXT)
                        .width(Length::Shrink),
                ]
                .width(Length::Shrink)
            },
        )
    };

    let metrics = container(
        row![
            metrics_label,
            iced::widget::space().width(16),
            metric_labels::view(),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .padding(iced::Padding::new(0.0).left(16.0).right(16.0))
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
