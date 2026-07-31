use iced::widget::{column, container, row, text};
use iced::{border, Element, Length};
use crate::modules::compute::metrics::Metrics;
use crate::modules::ui::line_chart::{LineChart, LineChartState};
use crate::modules::ui::theme;
use crate::modules::ui::volume_chart::VolumeChart;
use super::drawing_tools;
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

pub fn view<'a>(
    selected_halving: Option<u32>,
    yoy_selected: bool,
    halving_eta: Option<&str>,
    halving_subsidy: Option<&str>,
    chart_state: &'a LineChartState,
    metrics: &'a Metrics,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let placeholder_style = |_theme: &iced::Theme| -> container::Style {
        container::Style::default()
            .background(iced::Background::Color(theme::SIDEBAR_BACKGROUND))
            .border(border::rounded(8).color(theme::DASHBOARD_PLACEHOLDER_BORDER).width(1.5))
    };

    // The chart renders for YOY and for any selected halving. Only when no
    // page is active do we show the empty placeholder panels.
    let page_active = yoy_selected || selected_halving.is_some();

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
                .width(Length::Fill),
            text("Over")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Fill),
            text("Year")
                .size(18)
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Fill),
        ]
        .width(Length::Fixed(100.0))
    } else {
        selected_halving.map_or(
            iced::widget::column![].width(Length::Fixed(100.0)),
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
                        .width(Length::Fill),
                    text("HALVING")
                        .size(18)
                        .font(iced::Font {
                            family: iced::font::Family::Name("Geist Mono"),
                            weight: iced::font::Weight::Normal,
                            stretch: iced::font::Stretch::Normal,
                            style: iced::font::Style::Normal,
                        })
                        .color(theme::HALVING_BUTTON_TEXT)
                        .width(Length::Fill),
                ]
                .width(Length::Fixed(100.0))
            },
        )
    };

    let metrics = container(
        row![
            metrics_label,
            iced::widget::space().width(16),
            metric_labels::view(
                metrics,
                crate::modules::ui::mainwindow::application::Message::CalmarClicked,
            ),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .padding(iced::Padding::new(0.0).left(16.0).right(16.0))
    .style(placeholder_style);

    // A future halving page has no data, so show its ETA instead of a chart.
    let price: Element<'a, crate::modules::ui::mainwindow::application::Message> =
        if page_active && !chart_state.candles.is_empty() {
            let chart = container(LineChart::new(chart_state))
                .width(Length::Fill)
                .height(Length::FillPortion(7))
                .style(placeholder_style);

            let tools = container(
                drawing_tools::view(chart_state.drawing_mode.get())
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::End)
            .align_y(iced::Alignment::Start)
            .padding(iced::Padding::new(8.0));

            iced::widget::stack(vec![chart.into(), tools.into()]).into()
        } else if page_active {
            // Future halving: show the ETA and subsidy centered in the area.
            let ordinal = selected_halving.map(|n| format!("{}{}", n, ordinal_suffix(n)));
            let heading = ordinal.as_deref().unwrap_or("");
            let eta_text = halving_eta.unwrap_or("\u{2014}");
            let subsidy_text = halving_subsidy.unwrap_or("\u{2014}");
            container(
                column![
                    text(format!(
                        "{} HALVING",
                        heading,
                    ))
                    .size(16)
                    .font(iced::Font::with_name("Geist Mono"))
                    .color(theme::HALVING_BUTTON_TEXT),
                    text(format!("ETA \u{2014} {}", eta_text))
                        .size(14)
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                    text(format!("SUBSIDY \u{2014} {}", subsidy_text))
                        .size(14)
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::FillPortion(7))
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .style(placeholder_style)
            .into()
        } else {
            container(iced::widget::column![])
                .width(Length::Fill)
                .height(Length::FillPortion(7))
                .style(placeholder_style)
                .into()
        };

    let volume: Element<'a, crate::modules::ui::mainwindow::application::Message> = if page_active {
        container(VolumeChart::new(chart_state))
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .style(placeholder_style)
            .into()
    } else {
        container(iced::widget::column![])
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .style(placeholder_style)
            .into()
    };

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