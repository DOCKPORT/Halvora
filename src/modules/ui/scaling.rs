use iced::Size;

/// The reference resolution that all pixel values are designed for.
/// On a 1920×1080 screen, `sp(value)` returns the same value.
/// On larger/smaller screens, values are scaled proportionally.
const REFERENCE_WIDTH: f32 = 1920.0;
const REFERENCE_HEIGHT: f32 = 1080.0;

/// Detects the primary monitor's resolution.
///
/// Platform-specific:
/// - Linux: queries `xrandr` for the actual resolution
/// - Windows/macOS: returns a placeholder (maximized mode is used instead)
fn detect_screen_size() -> Size {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("xrandr").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(" primary") || line.contains('*') {
                    if let Some(res) = line
                        .split_whitespace()
                        .find(|s| {
                            s.contains('x')
                                && s.chars().all(|c| c.is_ascii_digit() || c == 'x')
                        }) {
                        let parts: Vec<&str> = res.split('x').collect();
                        if parts.len() == 2 {
                            if let (Ok(w), Ok(h)) =
                                (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                            {
                                return Size::new(w, h);
                            }
                        }
                    }
                }
            }
        }
        Size::new(1024.0, 768.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Size::new(1024.0, 768.0) // maximized mode handles sizing instead
    }
}

/// Lazily-computed scale factor and screen size.
///
/// Computed once on first access via [`Scaling::global`].
#[allow(dead_code)]
pub struct Scaling {
    /// The detected screen size.
    pub screen_size: Size,
    /// The scale factor (screen width / 1920).
    pub factor: f32,
}

impl Scaling {
    /// Returns the global [`Scaling`] instance, computing it on first call.
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<Scaling> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| {
            let screen_size = detect_screen_size();

            // Use width-based scaling, but also ensure height doesn't get squished
            let factor_x = screen_size.width / REFERENCE_WIDTH;
            let factor_y = screen_size.height / REFERENCE_HEIGHT;
            let factor = factor_x.min(factor_y);

            Scaling { screen_size, factor }
        })
    }

    /// Scales a pixel value from the reference resolution to the current screen.
    ///
    /// Usage: `sp(400)` returns 400 × scale_factor.
    pub fn sp(&self, value: f32) -> f32 {
        value * self.factor
    }
}

/// Convenience shorthand: scales a pixel value to the current screen.
///
/// Equivalent to `Scaling::global().sp(value)`.
pub fn sp(value: f32) -> f32 {
    Scaling::global().sp(value)
}