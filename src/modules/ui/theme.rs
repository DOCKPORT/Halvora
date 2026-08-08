use iced::Color;
use iced::widget::scrollable;

pub const MAINWINDOW_BACKGROUND: Color = Color::from_rgb(
    0x6c as f32 / 255.0,
    0x6c as f32 / 255.0,
    0x6c as f32 / 255.0,
);

pub const SIDEBAR_BACKGROUND: Color = Color::from_rgb(
    0x2a as f32 / 255.0,
    0x2a as f32 / 255.0,
    0x2a as f32 / 255.0,
);

pub const HALVING_BUTTON_BACKGROUND: Color = Color::from_rgb(
    0x3a as f32 / 255.0,
    0x3a as f32 / 255.0,
    0x3a as f32 / 255.0,
);

pub const HALVING_BUTTON_HOVER: Color = Color::from_rgb(
    0x4a as f32 / 255.0,
    0x4a as f32 / 255.0,
    0x4a as f32 / 255.0,
);

pub const HALVING_BUTTON_TEXT: Color = Color::from_rgb(
    0xcc as f32 / 255.0,
    0xcc as f32 / 255.0,
    0xcc as f32 / 255.0,
);

/// Sidebar button fills keyed to the period's P/L sign, matching the chart
/// line colors used elsewhere in the UI. Fully opaque, using darker shades so
/// the subtle dark-on-grey look is preserved.
pub const BUTTON_FILL_GREEN: Color = Color::from_rgb(0.0, 0.4, 0.15);
pub const BUTTON_FILL_GREEN_HOVER: Color = Color::from_rgb(0.0, 0.5, 0.19);
pub const BUTTON_FILL_RED: Color = Color::from_rgb(0.55, 0.05, 0.02);
pub const BUTTON_FILL_RED_HOVER: Color = Color::from_rgb(0.7, 0.07, 0.03);

pub const DASHBOARD_PLACEHOLDER_BORDER: Color = Color::from_rgb(
    0xf5 as f32 / 255.0,
    0xb3 as f32 / 255.0,
    0x42 as f32 / 255.0,
);

/// Splash screen backdrop, consistent with the dark UI theme.
pub const SPLASH_BACKGROUND: Color = Color::from_rgb(
    0x1a as f32 / 255.0,
    0x1a as f32 / 255.0,
    0x1a as f32 / 255.0,
);

/// Splash screen accent, matching the logo's orange.
pub const SPLASH_ACCENT: Color = Color::from_rgb(
    0xf5 as f32 / 255.0,
    0xb3 as f32 / 255.0,
    0x42 as f32 / 255.0,
);

/// Design width (in pixels at the 1920×1080 reference) of the slim sidebar
/// scrollbar. The default iced scrollbar is 10 px; this is intentionally
/// thinner.
const SIDEBAR_SCROLLBAR_WIDTH: f32 = 6.0;

/// Shared scrollable style for the sidebars.
///
/// Keeps the exact iced default appearance. The reduced scrollbar width is
/// applied separately by [`sidebar_scrollbar_direction`].
pub fn sidebar_scrollable_style(
    theme: &iced::Theme,
    status: scrollable::Status,
) -> scrollable::Style {
    scrollable::default(theme, status)
}

/// The slim vertical scrollbar direction used by the sidebars.
///
/// Both the rail and the scroller use the reduced `SIDEBAR_SCROLLBAR_WIDTH` so
/// they stay visually consistent.
pub fn sidebar_scrollbar_direction() -> scrollable::Direction {
    scrollable::Direction::Vertical(
        scrollable::Scrollbar::default()
            .width(crate::modules::ui::scaling::sp(SIDEBAR_SCROLLBAR_WIDTH))
            .scroller_width(crate::modules::ui::scaling::sp(SIDEBAR_SCROLLBAR_WIDTH)),
    )
}
