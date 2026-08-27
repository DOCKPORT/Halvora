//! The "Position" dialog, opened by clicking the Position card in the
//! blockchain sidebar.
//!
//! Lets the user enter their BTC balance and average DCA price. The dialog
//! previews the resulting P/L at the current price; Save persists the values
//! to the user position database (`user_position.db`).

use crate::modules::ui::mainwindow::application::Message;
use crate::modules::ui::mainwindow::dialog_chrome;
use crate::modules::ui::scaling::sp;
use iced::widget::{button, column, row, text, text_input};
use iced::{Color, Element, border};

/// The monospaced font used for all dialog text.
const FONT: iced::Font = iced::Font::with_name("Geist Mono");

/// Green for a positive preview, matching the chart's positive line.
const POSITIVE: Color = Color::from_rgb(0.0, 0.8, 0.3);

/// Red for a negative preview or an invalid input, matching the chart's
/// negative line.
const NEGATIVE: Color = Color::from_rgb(1.0, 0.1, 0.05);

/// Muted grey for labels and hints.
const MUTED: Color = Color::from_rgb(0.7, 0.7, 0.7);

/// Dark style for the dialog's text inputs, matching the rest of the UI.
fn input_style(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: iced::Background::Color(Color::from_rgb(0.1, 0.1, 0.1)),
        border: border::rounded(6)
            .color(Color::from_rgb(0.3, 0.3, 0.3))
            .width(1.0),
        icon: Color::from_rgb(0.6, 0.6, 0.6),
        placeholder: Color::from_rgb(0.5, 0.5, 0.5),
        value: Color::WHITE,
        selection: Color::from_rgb(0.3, 0.3, 0.8),
    }
}

/// Shared style for the dialog's Save and Close buttons.
fn dialog_button_style(
    normal: Color,
    hover: Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| button::Style {
        background: Some(iced::Background::Color(match status {
            button::Status::Hovered => hover,
            _ => normal,
        })),
        border: border::rounded(6),
        text_color: Default::default(),
        shadow: Default::default(),
        snap: false,
    }
}

/// Render the Position dialog as a full-screen dimmed overlay.
///
/// `balance_input` and `dca_input` are the draft values the user is editing.
/// `live_price` feeds the P/L preview. The Save button stays disabled while
/// the inputs are not valid non-negative numbers (≥ 0).
pub fn view<'a>(
    balance_input: &'a str,
    dca_input: &'a str,
    live_price: Option<f64>,
) -> Element<'a, Message> {
    // Validate the draft values so Save can be disabled early and the hint
    // text can guide the user before they submit. Both values may be zero, so
    // a position can be cleared completely.
    let balance: Option<f64> = balance_input.trim().parse().ok();
    let dca: Option<f64> = dca_input.trim().parse().ok();
    let valid = matches!(
        (balance, dca),
        (Some(b), Some(d)) if b.is_finite() && d.is_finite() && b >= 0.0 && d >= 0.0
    );

    // Live preview of the P/L the saved position would show at the current
    // price. An em-dash appears when the DCA is not usable yet.
    let preview = match (dca, live_price) {
        (Some(d), Some(p)) if d > 0.0 => {
            let change = (p - d) / d * 100.0;
            let (arrow, color) = if change >= 0.0 {
                ("\u{25B2}", POSITIVE)
            } else {
                ("\u{25BC}", NEGATIVE)
            };
            text(format!(
                "P/L at ${}: {arrow} {:.2}%",
                crate::modules::ui::ws_flash::format_usd(p),
                change.abs()
            ))
            .size(sp(16.0))
            .color(color)
            .font(FONT)
        }
        _ => text("\u{2014}").size(sp(16.0)).color(MUTED).font(FONT),
    };

    // A hint appears only when the user has typed something invalid. A fresh
    // dialog stays clean instead of showing an error on first open.
    let attempted = !balance_input.trim().is_empty() || !dca_input.trim().is_empty();
    let hint = if valid || !attempted {
        None
    } else {
        Some(
            text("Enter non-negative values for both fields.")
                .size(sp(13.0))
                .color(NEGATIVE)
                .font(FONT),
        )
    };

    let inner = column![
        text("Position")
            .size(sp(24.0))
            .color(Color::WHITE)
            .font(FONT),
        text("Enter your BTC balance and DCA.")
            .size(sp(15.0))
            .color(MUTED)
            .font(FONT),
        text("BTC Balance").size(sp(15.0)).color(MUTED).font(FONT),
        text_input("0.00000000", balance_input)
            .on_input(Message::PositionBalanceChanged)
            .font(FONT)
            .size(sp(17.0))
            .padding(iced::Padding::new(sp(8.0)).horizontal(sp(10.0)))
            .style(input_style),
        text("DCA Price (USD)")
            .size(sp(15.0))
            .color(MUTED)
            .font(FONT),
        text_input("0.00", dca_input)
            .on_input(Message::PositionDcaChanged)
            .font(FONT)
            .size(sp(17.0))
            .padding(iced::Padding::new(sp(8.0)).horizontal(sp(10.0)))
            .style(input_style),
        preview,
        if let Some(hint) = hint {
            hint
        } else {
            text("").size(sp(13.0))
        },
        row![
            button(text("Save").size(sp(16.0)).color(Color::WHITE))
                .on_press_maybe(valid.then_some(Message::SavePosition))
                .padding(iced::Padding::new(sp(8.0)).horizontal(sp(20.0)))
                .style(dialog_button_style(
                    Color::from_rgb(0.15, 0.45, 0.25),
                    Color::from_rgb(0.19, 0.55, 0.31),
                )),
            button(text("Close").size(sp(16.0)).color(Color::WHITE))
                .on_press(Message::ClosePositionDialog)
                .padding(iced::Padding::new(sp(8.0)).horizontal(sp(20.0)))
                .style(dialog_button_style(
                    Color::from_rgb(0.3, 0.3, 0.3),
                    Color::from_rgb(0.38, 0.38, 0.38),
                )),
        ]
        .spacing(sp(12.0)),
    ]
    .spacing(sp(12.0))
    .align_x(iced::Alignment::Center)
    .padding(sp(32.0));

    dialog_chrome::overlay(dialog_chrome::card(inner.into(), sp(420.0)))
}
