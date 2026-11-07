//! The "Brass Tacks" dialog, opened by the Brass Tacks sidebar button.
//!
//! Styled to match the Calmar dialog: a dim full-screen overlay with a
//! centered, rounded card. Compares the performance of Bitcoin and the
//! S&P 500 since August 18, 2011.

use crate::modules::compute::brass_tacks;
use crate::modules::ui::mainwindow::application::Message;
use crate::modules::ui::mainwindow::dialog_chrome;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::ws_flash::{FLASH_DOWN, FLASH_UP, format_usd};
use iced::widget::{button, row, text};
use iced::{Color, Element, Length};

/// The monospaced font used for all dialog text.
const FONT: iced::Font = iced::Font::with_name("Geist Mono");

/// Format a price as `$1,234.56`, or an em-dash when missing.
fn fmt_price(value: Option<f64>) -> String {
    match value {
        Some(p) => format!("${}", format_usd(p)),
        None => "\u{2014}".to_string(),
    }
}

/// Build a "Change:" line with a colored direction arrow and a percent value.
///
/// The arrow is green when the change is positive, red when it is negative,
/// and white for zero or missing data. The value uses thousands separators.
fn change_row<'a>(change: Option<f64>) -> Element<'a, Message> {
    let (arrow, color, value) = match change {
        Some(c) if c > 0.0 => ("\u{2191}", FLASH_UP, format!("{}%", format_usd(c))),
        Some(c) if c < 0.0 => ("\u{2193}", FLASH_DOWN, format!("{}%", format_usd(-c))),
        Some(_) => ("\u{25CF}", Color::WHITE, "0.00%".to_string()),
        None => ("\u{2014}", Color::WHITE, String::new()),
    };

    row![
        text("Change:  ")
            .size(sp(16.0))
            .color(Color::WHITE)
            .font(FONT),
        text(arrow).size(sp(16.0)).color(color).font(FONT),
        text(format!(" {value}"))
            .size(sp(16.0))
            .color(Color::WHITE)
            .font(FONT),
    ]
    .into()
}

/// Build one asset block: its name, the price change from start to now, and
/// the percent change row.
fn asset_block<'a>(
    name: &'a str,
    start: f64,
    current: Option<f64>,
    pct: Option<f64>,
) -> Element<'a, Message> {
    iced::widget::column![
        text(name).size(sp(18.0)).color(Color::WHITE).font(FONT),
        text(format!(
            "{}  \u{2192}  {}",
            fmt_price(Some(start)),
            fmt_price(current)
        ))
        .size(sp(16.0))
        .color(Color::WHITE)
        .font(FONT),
        change_row(pct),
    ]
    .spacing(sp(12.0))
    .align_x(iced::Alignment::Center)
    .into()
}

/// Split the BTC vs S&P 500 result into a label and a percentage value.
///
/// The label names the outcome and the value holds the size of the gap. The
/// value is empty when there is no gap or no data.
fn outperformance_parts(delta: Option<f64>) -> (String, String) {
    match delta {
        Some(d) if d > 0.0 => (
            "Bitcoin outperforms the S&P 500 by".to_string(),
            format!("{}%", format_usd(d)),
        ),
        Some(d) if d < 0.0 => (
            "Bitcoin underperforms the S&P 500 by".to_string(),
            format!("{}%", format_usd(-d)),
        ),
        Some(_) => (
            "Bitcoin and the S&P 500 performed the same".to_string(),
            String::new(),
        ),
        None => ("\u{2014}".to_string(), String::new()),
    }
}

/// Render the Brass Tacks dialog as a full-screen dimmed overlay.
///
/// `btc_current` is the live websocket price and `sp500_current` is the
/// fetched S&P 500 value. The start prices and the start date are constants.
/// The caller stacks this on top of the main content (wrapped in a
/// `mouse_area`) so the modal blocks interaction behind it.
pub fn view<'a>(
    btc_current: Option<f64>,
    sp500_current: Option<f64>,
) -> Element<'a, Message> {
    let btc_start = brass_tacks::BTC_START_VALUE;
    let sp500_start = brass_tacks::SP500_START_VALUE;

    // Percentage change for each asset from its start price to now.
    let btc_pct = btc_current.and_then(|current| brass_tacks::pct_change(btc_start, current));
    let sp500_pct = sp500_current.and_then(|current| brass_tacks::pct_change(sp500_start, current));

    // How far BTC outperforms (positive) or underperforms (negative).
    let delta = brass_tacks::outperformance(btc_pct, sp500_pct);
    let (outcome_label, outcome_value) = outperformance_parts(delta);

    let inner = iced::widget::column![
        text("Bitcoin vs S&P 500")
            .size(sp(16.0))
            .color(Color::WHITE)
            .font(FONT),
        text(format!("Since {}", brass_tacks::START_DATE_LABEL))
            .size(sp(15.0))
            .color(Color::WHITE)
            .font(FONT),
        text("\u{2500}").size(sp(14.0)).color(Color::WHITE).font(FONT),
        asset_block("Bitcoin", btc_start, btc_current, btc_pct),
        text("\u{2500}").size(sp(14.0)).color(Color::WHITE).font(FONT),
        asset_block("S&P 500", sp500_start, sp500_current, sp500_pct),
        text("\u{2500}").size(sp(14.0)).color(Color::WHITE).font(FONT),
        text(outcome_label)
            .size(sp(18.0))
            .color(Color::WHITE)
            .font(FONT),
        text(outcome_value)
            .size(sp(24.0))
            .color(Color::WHITE)
            .font(FONT),
        iced::widget::space().height(Length::Fixed(sp(12.0))),
        button(text("Close").size(sp(16.0)).color(Color::WHITE))
            .on_press(Message::CloseBrassTacksDialog)
            .padding(iced::Padding::new(sp(8.0)).horizontal(sp(16.0)))
            .style(|_theme, _status| button::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
                border: iced::border::rounded(6),
                shadow: Default::default(),
                text_color: Default::default(),
                snap: false,
            }),
    ]
    .spacing(sp(12.0))
    .width(Length::Fill)
    .align_x(iced::Alignment::Center)
    .padding(sp(32.0));

    dialog_chrome::overlay(dialog_chrome::card(inner.into(), sp(440.0)))
}
