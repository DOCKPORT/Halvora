//! The "Calmar Ratio Details" dialog, opened by clicking the Calmar metric.
//!
//! Styled to match the About dialog: a dim full-screen overlay with a
//! centered, rounded card. Shows the breakdown behind the Calmar ratio.

use crate::modules::compute::metrics::Metrics;
use crate::modules::ui::mainwindow::application::Message;
use crate::modules::ui::mainwindow::dialog_chrome;
use crate::modules::ui::scaling::sp;
use iced::widget::{button, text};
use iced::{Color, Element};

/// Render the Calmar ratio details dialog as a full-screen dimmed overlay.
///
/// The caller stacks this on top of the main content (wrapped in a
/// `mouse_area`) so the modal blocks interaction behind it.
pub fn view(metrics: &Metrics) -> Element<'_, Message> {
    let breakdown = &metrics.calmar_breakdown;
    let inner = iced::widget::column![
        text("Calmar Ratio Details")
            .size(sp(20.0))
            .color(Color::WHITE)
            .font(iced::Font::with_name("Geist Mono")),
        text("─")
            .size(sp(12.0))
            .color(Color::from_rgb(0.4, 0.4, 0.4))
            .font(iced::Font::with_name("Geist Mono")),
        text("Formula: Annualized Return ÷ Max Drawdown")
            .size(sp(13.0))
            .color(Color::from_rgb(0.8, 0.8, 0.8))
            .font(iced::Font::with_name("Geist Mono")),
        text("• Daily P/L%: (Close − Open) / Open")
            .size(sp(12.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text("• Weighted Avg: Σ(P/L% × Vol) / Σ(Vol)")
            .size(sp(12.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text("• Annualized: Weighted Avg × 365")
            .size(sp(12.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text("• Ratio: Annualized / Max DD")
            .size(sp(12.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text("─")
            .size(sp(12.0))
            .color(Color::from_rgb(0.4, 0.4, 0.4))
            .font(iced::Font::with_name("Geist Mono")),
        text(format!("Weighted Avg P/L:  {}", breakdown.weighted_avg_pl))
            .size(sp(14.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text(format!(
            "Annualized Return:  {}",
            breakdown.annualized_return
        ))
        .size(sp(14.0))
        .color(Color::from_rgb(0.7, 0.7, 0.7))
        .font(iced::Font::with_name("Geist Mono")),
        text(format!("Max Drawdown:  {}", breakdown.max_drawdown))
            .size(sp(14.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        text(format!("Calmar Ratio:  {}", breakdown.ratio))
            .size(sp(14.0))
            .color(Color::from_rgb(0.7, 0.7, 0.7))
            .font(iced::Font::with_name("Geist Mono")),
        button(text("Close").size(sp(14.0)).color(Color::WHITE))
            .on_press(Message::CloseCalmarDialog)
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
    .align_x(iced::Alignment::Center)
    .padding(sp(32.0));

    dialog_chrome::overlay(dialog_chrome::card(inner.into(), sp(400.0)))
}
