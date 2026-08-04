use iced::widget::{column, container, progress_bar, svg};
use iced::{Color, Element, Length, Vector};
use crate::modules::ui::theme;

use super::state::SplashState;

/// The banner SVG used by the splash screen, embedded in the binary at
/// compile time so the running program does not depend on any path on disk.
/// It contains the "HALVORA" wordmark and the circular logo mark.
const LOGO_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/Halvora_Logo/Halvora.svg"
));

/// Width of the rendered banner, scaled to the current screen. The height is
/// derived from the banner's wide aspect ratio via ContentFit::Contain.
const LOGO_WIDTH: f32 = 420.0;

/// Renders the splash screen: centered banner with a drop shadow and a
/// progress bar. The splash fades out before the transition to the main
/// dashboard, driven by `SplashState::opacity`.
pub fn view<'a>(state: &'a SplashState) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    // Current fade opacity in the range 0.0..=1.0.
    let opacity = state.opacity();

    let logo_svg = svg(svg::Handle::from_memory(LOGO_SVG))
        .width(Length::Fixed(crate::modules::ui::scaling::sp(LOGO_WIDTH)))
        .opacity(opacity);

    // Wrap the banner in a container that carries a soft drop shadow.
    let banner = container(logo_svg)
        .style(move |_theme| container::Style {
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35).scale_alpha(opacity),
                offset: Vector::new(0.0, 6.0),
                blur_radius: crate::modules::ui::scaling::sp(20.0),
            },
            ..Default::default()
        });

    let bar = progress_bar(0.0..=1.0, state.progress())
        .length(Length::Fixed(crate::modules::ui::scaling::sp(LOGO_WIDTH)))
        .girth(Length::Fixed(crate::modules::ui::scaling::sp(6.0)))
        .style(move |_theme: &iced::Theme| progress_bar::Style {
            background: iced::Background::Color(
                Color::from_rgba(1.0, 1.0, 1.0, 0.15).scale_alpha(opacity),
            ),
            bar: iced::Background::Color(theme::SPLASH_ACCENT.scale_alpha(opacity)),
            border: iced::border::rounded(4),
        });

    let content = column![banner, bar]
        .spacing(crate::modules::ui::scaling::sp(24.0))
        .align_x(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(
                theme::SPLASH_BACKGROUND.scale_alpha(opacity),
            )),
            ..Default::default()
        })
        .into()
}