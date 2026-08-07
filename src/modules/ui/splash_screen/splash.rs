use crate::modules::ui::theme;
use iced::widget::{Space, column, container, progress_bar, stack, svg};
use iced::{Color, Element, Length, Vector};

use super::crosshatch_background;
use super::state::SplashState;

/// The banner SVG used by the splash screen, embedded in the binary at
/// compile time so the running program does not depend on any path on disk.
/// It contains the "HALVORA" wordmark and the circular logo mark.
const LOGO_SVG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/Halvora_Logo/Halvora.svg"
));

/// Width of the rendered banner, scaled to the current screen. The height is
/// derived from the banner's wide aspect ratio via `ContentFit::Contain`.
const LOGO_WIDTH: f32 = 420.0;

/// Uniform padding around the logo and progress bar that forms the solid
/// rectangle placed behind them, in unscaled reference pixels.
const BACKDROP_PADDING: f32 = 24.0;

/// Renders the splash screen: centered banner with a drop shadow and a
/// progress bar. The splash fades out before the transition to the main
/// dashboard, driven by `SplashState::opacity`.
pub fn view(
    state: &SplashState,
) -> Element<'_, crate::modules::ui::mainwindow::application::Message> {
    // Hold a completely empty frame until the true window size (and thus the
    // scale factor) is known. Rendering nothing avoids any size-dependent
    // content at the wrong scale, which would cause a startup jump.
    if !state.is_ready() {
        return Space::new().width(Length::Fill).height(Length::Fill).into();
    }

    // Current fade opacity in the range 0.0..=1.0.
    let opacity = state.opacity();

    let logo_svg = svg(svg::Handle::from_memory(LOGO_SVG))
        .width(Length::Fixed(crate::modules::ui::scaling::sp(LOGO_WIDTH)))
        .opacity(opacity);

    // Wrap the banner in a container that carries a soft drop shadow.
    let banner = container(logo_svg).style(move |_theme| container::Style {
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

    // A solid rectangle in the background colour sits directly behind the
    // banner and progress bar, hiding the cross-hatch lines there so the
    // logo reads cleanly. The cross-hatch stays visible around the edges.
    let backdrop_padding = crate::modules::ui::scaling::sp(BACKDROP_PADDING);
    let backdrop = container(content)
        .padding(backdrop_padding)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(
                theme::SPLASH_BACKGROUND.scale_alpha(opacity),
            )),
            border: iced::border::rounded(crate::modules::ui::scaling::sp(8.0)),
            ..Default::default()
        });

    // `Stack` places all children on top of each other but aligns them to
    // the top-left by default, so wrap the backdrop in a full-size container
    // that centers it.
    let centered_backdrop = container(backdrop)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center);

    let layered = stack![crosshatch_background::view(opacity), centered_backdrop];

    container(layered)
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
