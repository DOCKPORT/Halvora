//! Shared visual chrome for the modal dialogs (About and Calmar).
//!
//! Both dialogs use the same dim full-screen overlay with a centered,
//! rounded card. Keeping the chrome here avoids duplicating it.

use iced::widget::container;
use iced::{Color, Element, Length};

/// Wrap `inner` in the centered, rounded dialog card at `width` px.
pub fn card<'a, Message>(inner: Element<'a, Message>, width: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    container(inner)
        .width(Length::Fixed(width))
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            border: iced::border::rounded(12)
                .color(Color::from_rgb(0.3, 0.3, 0.3))
                .width(1.5),
            ..Default::default()
        })
        .into()
}

/// Wrap `content` in the dim full-screen overlay that centers it.
pub fn overlay<'a, Message>(content: Element<'a, Message>) -> Element<'a, Message>
where
    Message: 'a,
{
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                0.0, 0.0, 0.0, 0.6,
            ))),
            ..Default::default()
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
