use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use crate::modules::ui::ws_flash::{self, WsFlash};
use iced::widget::{Column, container, row, scrollable, stack, text};
use iced::{Color, Element, Length};

/// Format a raw mining difficulty as a compact value, for example
/// `126231507121868.2` becomes `126.23T`. Returns an em-dash when the value
/// is non-positive (no data yet).
fn fmt_difficulty(v: f64) -> String {
    if v <= 0.0 {
        return "\u{2014}".to_string();
    }
    if v >= 1_000_000_000_000.0 {
        format!("{:.2}T", v / 1_000_000_000_000.0)
    } else if v >= 1_000_000_000.0 {
        format!("{:.2}B", v / 1_000_000_000.0)
    } else if v >= 1_000_000.0 {
        format!("{:.2}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.2}K", v / 1_000.0)
    } else {
        format!("{v:.2}")
    }
}

fn value_text<'a>(
    content: String,
    color: Color,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    text(content)
        .size(sp(16.0))
        .font(iced::Font {
            family: iced::font::Family::Name("Geist Mono"),
            weight: iced::font::Weight::Semibold,
            stretch: iced::font::Stretch::Normal,
            style: iced::font::Style::Normal,
        })
        .color(color)
        .into()
}

fn info_card<'a>(
    title: &'a str,
    value: Element<'a, crate::modules::ui::mainwindow::application::Message>,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    let inner = Column::with_children(vec![
        text(title)
            .size(sp(14.0))
            .color(theme::HALVING_BUTTON_TEXT)
            .into(),
        value,
    ])
    .spacing(sp(4.0))
    .padding(iced::Padding::new(sp(8.0)));

    container(inner)
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(theme::HALVING_BUTTON_BACKGROUND)),
            border: iced::border::rounded(8)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                .width(1.0),
            ..Default::default()
        })
        .into()
}

pub fn view<'a>(
    current_tip_height: u32,
    current_subsidy_sat: i64,
    mining_difficulty: f64,
    next_halving_eta: &str,
    blocks_to_next_halving: &str,
    coins_issued: &str,
    percentage_issued: &str,
    remaining_issuance: &str,
    live_price: Option<f64>,
    spot_flash: Option<&WsFlash>,
    subsidy_value: &str,
    sats_per_usd: &str,
    all_time_high: &str,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    fn fmt_commas(n: u64) -> String {
        let s = n.to_string();
        let mut result = String::with_capacity(s.len() + s.len() / 3);
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(c);
        }
        result
    }

    let height_str = fmt_commas(u64::from(current_tip_height));
    // Reuse the same formatter as the page metric labels so trailing zeros
    // are trimmed, e.g. "3.125 BTC" instead of "3.12500000".
    let subsidy_str =
        crate::modules::compute::halving_period::subsidy_btc_from_sat(current_subsidy_sat);

    // Spot Price value: "$" + unchanged prefix stay normal, and the changed
    // digits (the leftmost diff and everything to its right) flash green/red
    // while a websocket flash is active.
    let spot_value: Element<'_, crate::modules::ui::mainwindow::application::Message> =
        match live_price {
            Some(p) => {
                let number = ws_flash::format_usd(p);
                let full = format!("${number}");
                match spot_flash.and_then(crate::modules::ui::ws_flash::WsFlash::diff_index) {
                    Some(i) if i < number.len() => {
                        let color = spot_flash.map_or(
                            theme::HALVING_BUTTON_TEXT,
                            crate::modules::ui::ws_flash::WsFlash::color,
                        );
                        let prefix = format!("${}", &number[..i]);
                        let suffix = number[i..].to_string();
                        row![
                            value_text(prefix, theme::HALVING_BUTTON_TEXT),
                            value_text(suffix, color),
                        ]
                        .spacing(0)
                        .into()
                    }
                    _ => value_text(full, theme::HALVING_BUTTON_TEXT),
                }
            }
            None => value_text("\u{2014}".to_string(), theme::HALVING_BUTTON_TEXT),
        };

    let content = Column::with_children(vec![
        info_card("Spot Price", spot_value),
        info_card(
            "Block Height",
            value_text(height_str, theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Current Subsidy",
            value_text(subsidy_str, theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Mining Difficulty",
            value_text(
                fmt_difficulty(mining_difficulty),
                theme::HALVING_BUTTON_TEXT,
            ),
        ),
        info_card(
            "Subsidy Value",
            value_text(subsidy_value.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Next Halving",
            value_text(next_halving_eta.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Blocks to Halving",
            value_text(
                blocks_to_next_halving.to_string(),
                theme::HALVING_BUTTON_TEXT,
            ),
        ),
        info_card(
            "Coins Minted",
            value_text(coins_issued.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Percentage Issued",
            value_text(percentage_issued.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Remaining Issuance",
            value_text(remaining_issuance.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "Sats per USD",
            value_text(sats_per_usd.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
        info_card(
            "All-Time High",
            value_text(all_time_high.to_string(), theme::HALVING_BUTTON_TEXT),
        ),
    ])
    .spacing(sp(8.0))
    .padding(iced::Padding::new(sp(8.0)).left(sp(21.0)).right(sp(21.0)));

    // Cross-hatch lines sit just below the labels layer, behind the info
    // cards. `Stack` places all children on top of each other but aligns them
    // to the top-left by default, so wrap the scrollable in a full-size
    // container. The container is transparent so the cross-hatch shows through.
    let scrollable_layer: Element<'a, crate::modules::ui::mainwindow::application::Message> =
        container(
            scrollable(content)
                .direction(crate::modules::ui::theme::sidebar_scrollbar_direction())
                .style(crate::modules::ui::theme::sidebar_scrollable_style),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into();

    container(
        stack![
            crate::modules::ui::splash_screen::crosshatch_background::view_with_h_v_padding(
                0.35, 12.0, 0.0
            ),
            scrollable_layer,
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fixed(sp(250.0)))
    .height(Length::Fill)
    .padding(0)
    .style(|_theme| {
        container::Style::default().background(iced::Background::Color(theme::SIDEBAR_BACKGROUND))
    })
    .into()
}
