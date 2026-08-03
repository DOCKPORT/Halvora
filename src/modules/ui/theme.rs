use iced::Color;

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

/// Sidebar button tints keyed to the period's P/L sign, matching the chart
/// line colors used elsewhere in the UI. Applied at ~25% opacity so the dark
/// button still shows through.
pub const BUTTON_FILL_GREEN: Color = Color::from_rgba(0.0, 0.8, 0.3, 0.25);
pub const BUTTON_FILL_GREEN_HOVER: Color = Color::from_rgba(0.0, 0.8, 0.3, 0.4);
pub const BUTTON_FILL_RED: Color = Color::from_rgba(1.0, 0.1, 0.05, 0.25);
pub const BUTTON_FILL_RED_HOVER: Color = Color::from_rgba(1.0, 0.1, 0.05, 0.4);

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
