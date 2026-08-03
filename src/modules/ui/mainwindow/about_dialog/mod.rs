//! The "About Halvora" dialog, opened by clicking the logo in the sidebar.
//!
//! Styled to match the Calmar ratio details dialog: a dim full-screen overlay
//! with a centered, rounded card. Shows the app version, a short description,
//! and the GitHub icon as a clickable link to the source repository.

use iced::widget::{button, container, svg, text};
use iced::{Color, ContentFit, Element, Length};
use crate::modules::ui::mainwindow::application::Message;

/// Render the About dialog as a full-screen dimmed overlay.
///
/// The caller stacks this on top of the main content (wrapped in a
/// `mouse_area`) so the modal blocks interaction behind it.
pub fn view<'a>() -> Element<'a, Message> {
    let inner = iced::widget::column![
        text(format!("Version {}", env!("CARGO_PKG_VERSION")))
            .size(18)
            .color(Color::from_rgb(0.9, 0.9, 0.9))
            .font(iced::Font::with_name("Geist Mono")),
        text("\u{2500}")
            .size(12)
            .color(Color::from_rgb(0.4, 0.4, 0.4))
            .font(iced::Font::with_name("Geist Mono")),
        text(
            "Halvora tracks all 32 Bitcoin halvings and their price action. \
It provides block-height precision, anchored chart analysis, and \
performance metrics across every epoch, from genesis to the final \
block subsidy.",
        )
        .size(16)
        .color(Color::from_rgb(0.9, 0.9, 0.9))
        .font(iced::Font::with_name("Geist Mono")),
        text("\u{2500}")
            .size(12)
            .color(Color::from_rgb(0.4, 0.4, 0.4))
            .font(iced::Font::with_name("Geist Mono")),
        // The GitHub icon is the hyperlink; clicking it opens the repo.
        button(
            svg::Svg::new("Halvora_Logo/GitHub_Invertocat_White_Clearspace.svg")
                .content_fit(ContentFit::Contain)
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0)),
        )
        .padding(8)
        .on_press(Message::OpenGithub)
        .style(|_theme, status| button::Style {
            background: Some(iced::Background::Color(match status {
                button::Status::Hovered => Color::from_rgb(0.3, 0.3, 0.3),
                _ => Color::from_rgba(0.3, 0.3, 0.3, 0.0),
            })),
            border: iced::border::rounded(6),
            shadow: Default::default(),
            text_color: Default::default(),
            snap: false,
        }),
        button(
            text("Close")
                .size(14)
                .color(Color::WHITE),
        )
        .on_press(Message::CloseAboutDialog)
        .padding(iced::Padding::new(8.0).horizontal(16.0))
        .style(|_theme, _status| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
            border: iced::border::rounded(6),
            shadow: Default::default(),
            text_color: Default::default(),
            snap: false,
        }),
    ]
    .spacing(12)
    .align_x(iced::Alignment::Center)
    .padding(32);

    container(container(inner).width(Length::Fixed(440.0)).style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
        border: iced::border::rounded(12)
            .color(Color::from_rgb(0.3, 0.3, 0.3))
            .width(1.5),
        ..Default::default()
    }))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.6))),
        ..Default::default()
    })
    .align_x(iced::Alignment::Center)
    .align_y(iced::Alignment::Center)
    .into()
}
