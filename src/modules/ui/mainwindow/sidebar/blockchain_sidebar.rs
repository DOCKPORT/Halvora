use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use crate::modules::ui::ws_flash::{self, WsFlash};
use iced::widget::{Column, button, container, row, scrollable, stack, text};
use iced::{Color, Element, Length};

/// Text color for a positive position P/L, matching the metric bar.
const POSITION_UP: Color = Color::from_rgb(0.0, 0.8, 0.3);

/// Text color for a negative position P/L, matching the metric bar.
const POSITION_DOWN: Color = Color::from_rgb(1.0, 0.1, 0.05);

/// Text color for a position P/L with no usable data, matching the metric bar.
const POSITION_FLAT: Color = Color::from_rgb(0.5, 0.5, 0.5);

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
        .size(sp(17.0))
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
            .size(sp(15.0))
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

/// P/L percent of the stored position against the live price:
/// `(price - dca) / dca * 100`. `None` when no position is set, the balance
/// or DCA is non-positive, or no live price is available yet.
fn position_pl_pct(position: Option<(f64, f64)>, live_price: Option<f64>) -> Option<f64> {
    let (balance, dca) = position?;
    let price = live_price?;
    if balance <= 0.0 || dca <= 0.0 {
        return None;
    }
    Some((price - dca) / dca * 100.0)
}

/// The clickable "Position" card at the top of the blockchain sidebar. Shows
/// the stored position's P/L against the websocket price and opens the
/// Position dialog when clicked.
fn position_card<'a>(
    position: Option<(f64, f64)>,
    live_price: Option<f64>,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;

    // P/L percent using the ▲/▼ and color convention of the metric bar.
    let pl = position_pl_pct(position, live_price);
    let (pl_text, pl_color) = match pl {
        Some(change) if change >= 0.0 => (format!("\u{25B2} {change:.2}%"), POSITION_UP),
        Some(change) => (format!("\u{25BC} {:.2}%", -change), POSITION_DOWN),
        None => ("\u{2014}".to_string(), POSITION_FLAT),
    };

    // The card shows only the P/L against the websocket price, matching the
    // other info cards. The dialog holds the balance and DCA details.
    let inner = Column::with_children(vec![
        text("Position")
            .size(sp(15.0))
            .color(theme::HALVING_BUTTON_TEXT)
            .into(),
        value_text(pl_text, pl_color),
    ])
    .spacing(sp(4.0))
    .padding(iced::Padding::new(sp(8.0)));

    // `padding(0)` keeps the button's bounds identical to the info cards, so
    // the border hugs the content the same way. The hover fill matches the
    // Calmar card in the metric bar.
    button(inner)
        .width(Length::Fill)
        .padding(0)
        .on_press(Message::PositionClicked)
        .style(|_theme, status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => theme::HALVING_BUTTON_HOVER,
                _ => theme::HALVING_BUTTON_BACKGROUND,
            })),
            border: iced::border::rounded(8)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
                .width(1.0),
            text_color: Default::default(),
            shadow: Default::default(),
            snap: true,
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
    position: Option<(f64, f64)>,
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
        position_card(position, live_price),
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
