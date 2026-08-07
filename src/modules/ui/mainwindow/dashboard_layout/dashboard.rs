use iced::widget::{column, container, row, stack, text};
use iced::{border, Element, Length};
use crate::modules::compute::metrics::Metrics;
use crate::modules::ui::line_chart::{LineChart, LineChartState};
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use crate::modules::ui::volume_chart::VolumeChart;
use super::drawing_tools;
use super::metric_labels;

fn ordinal_suffix(n: u32) -> &'static str {
    match n % 100 {
        11..=13 => "TH",
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
    subsidy_label: &'a str,
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
                .size(sp(18.0))
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Fill),
            text("Over")
                .size(sp(18.0))
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Fill),
            text("Year")
                .size(sp(18.0))
                .font(iced::Font {
                    family: iced::font::Family::Name("Geist Mono"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                })
                .color(theme::HALVING_BUTTON_TEXT)
                .width(Length::Fill),
        ]
        .width(Length::Fixed(sp(100.0)))
    } else {
        selected_halving.map_or(
            iced::widget::column![].width(Length::Fixed(sp(100.0))),
            |n| {
                column![
                    text(format!("{}{}", n, ordinal_suffix(n)))
                        .size(sp(18.0))
                        .font(iced::Font {
                            family: iced::font::Family::Name("Geist Mono"),
                            weight: iced::font::Weight::Normal,
                            stretch: iced::font::Stretch::Normal,
                            style: iced::font::Style::Normal,
                        })
                        .color(theme::HALVING_BUTTON_TEXT)
                        .width(Length::Fill),
                    text("HALVING")
                        .size(sp(18.0))
                        .font(iced::Font {
                            family: iced::font::Family::Name("Geist Mono"),
                            weight: iced::font::Weight::Normal,
                            stretch: iced::font::Stretch::Normal,
                            style: iced::font::Style::Normal,
                        })
                        .color(theme::HALVING_BUTTON_TEXT)
                        .width(Length::Fill),
                ]
                .width(Length::Fixed(sp(100.0)))
            },
        )
    };

    let metrics = container(
        row![
            metrics_label,
            iced::widget::space().width(0),
            metric_labels::view(
                metrics,
                subsidy_label,
                crate::modules::ui::mainwindow::application::Message::CalmarClicked,
            ),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::FillPortion(1))
    .padding(iced::Padding::new(0.0).left(sp(16.0)).right(sp(16.0)))
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
            .padding(iced::Padding::new(sp(8.0)));

            // Block range label at the top-left of the placeholder box for a
            // started halving, layered above the chart.
            let block_label: Option<Element<'a, crate::modules::ui::mainwindow::application::Message>> =
                chart_state.block_range.get().map(|(start, end)| {
                    fn fmt_height(h: u64) -> String {
                        let s = h.to_string();
                        let mut result = String::with_capacity(s.len() + s.len() / 3);
                        for (i, c) in s.chars().enumerate() {
                            if i > 0 && (s.len() - i).is_multiple_of(3) {
                                result.push(',');
                            }
                            result.push(c);
                        }
                        result
                    }
                    container(
                        text(format!(
                            "BLOCK RANGE \u{2014} {} \u{2192} {}",
                            fmt_height(start),
                            fmt_height(end),
                        ))
                        .size(sp(14.0))
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::Alignment::Start)
                    .align_y(iced::Alignment::Start)
                    .padding(iced::Padding::new(sp(8.0)))
                    .into()
                });

            let mut price_layers: Vec<Element<'a, crate::modules::ui::mainwindow::application::Message>> =
                vec![chart.into(), tools.into()];
            if let Some(label) = block_label {
                price_layers.push(label);
            }
            iced::widget::stack(price_layers).into()
        } else if page_active {
            // Future halving (null halving): show the ETA and subsidy centered
            // over a cross-hatch placeholder in the splash's style.
            let ordinal = selected_halving.map(|n| format!("{}{}", n, ordinal_suffix(n)));
            let heading = ordinal.as_deref().unwrap_or("");
            let eta_text = halving_eta.unwrap_or("\u{2014}");
            let subsidy_text = halving_subsidy.unwrap_or("\u{2014}");

            // Block range for the selected halving, e.g. H-5 -> "1,050,000 → 1,260,000".
            fn fmt_height(h: u64) -> String {
                let s = h.to_string();
                let mut result = String::with_capacity(s.len() + s.len() / 3);
                for (i, c) in s.chars().enumerate() {
                    if i > 0 && (s.len() - i).is_multiple_of(3) {
                        result.push(',');
                    }
                    result.push(c);
                }
                result
            }
            let block_range_text = selected_halving
                .and_then(crate::modules::compute::halving_period::halving_block_range)
                .map(|(start, end)| format!("{} \u{2192} {}", fmt_height(start), fmt_height(end)))
                .unwrap_or_else(|| "\u{2014}".to_string());

            let info_column: Element<'a, crate::modules::ui::mainwindow::application::Message> =
                column![
                    text(format!(
                        "{} HALVING",
                        heading,
                    ))
                    .size(sp(22.0))
                    .font(iced::Font::with_name("Geist Mono"))
                    .color(theme::HALVING_BUTTON_TEXT),
                    text(format!("ETA \u{2014} {}", eta_text))
                        .size(sp(18.0))
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                    text(format!("SUBSIDY \u{2014} {}", subsidy_text))
                        .size(sp(18.0))
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                    text(format!("BLOCK RANGE \u{2014} {}", block_range_text))
                        .size(sp(18.0))
                        .font(iced::Font::with_name("Geist Mono"))
                        .color(theme::HALVING_BUTTON_TEXT),
                ]
                .spacing(sp(8.0))
                .align_x(iced::Alignment::Center)
                .into();

            // A solid rectangle in the placeholder's background colour sits
            // directly behind the text, hiding the cross-hatch there so the
            // labels read cleanly. Same corner radius as the splash backdrop.
            let backdrop: Element<'a, crate::modules::ui::mainwindow::application::Message> =
                container(info_column)
                    .padding(sp(24.0))
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(theme::SIDEBAR_BACKGROUND)),
                        border: iced::border::rounded(sp(8.0)),
                        ..Default::default()
                    })
                    .into();

            // `Stack` pins children to the top-left by default, so wrap the
            // backdrop in a full-size container that centers it.
            let centered_info: Element<'a, crate::modules::ui::mainwindow::application::Message> =
                container(backdrop)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
                    .into();

            container(
                stack![
                    crate::modules::ui::splash_screen::crosshatch_background::view_with_padding(1.0, 20.0),
                    centered_info,
                ]
                .width(Length::Fill)
                .height(Length::Fill),
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

    let volume: Element<'a, crate::modules::ui::mainwindow::application::Message> = if page_active
        && chart_state.candles.is_empty()
    {
        // Null halving: no volume data yet, so show only the cross-hatch
        // placeholder in the splash's style (below the line-chart text).
        let crosshatch: Element<'a, crate::modules::ui::mainwindow::application::Message> =
            crate::modules::ui::splash_screen::crosshatch_background::view_with_padding(1.0, 20.0);

        container(crosshatch)
            .width(Length::Fill)
            .height(Length::FillPortion(2))
            .style(placeholder_style)
            .into()
    } else if page_active {
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
            .spacing(sp(16.0))
            .padding(sp(16.0))
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