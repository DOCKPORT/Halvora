use iced::widget::row;
use iced::{Element, window, Font, Length};
use iced::window::Position;
use crate::modules::ui::scaling::Scaling;
use crate::modules::ui::mainwindow::dashboard_layout::dashboard;
use crate::modules::ui::mainwindow::sidebar::halving_sidebar;
use crate::modules::ui::mainwindow::sidebar::blockchain_sidebar;

/// Embed the GeistMono font as fallback — the system-installed SemiBold
/// variant will be used via the Font weight setting.
const GEIST_MONO_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/Font/geist-font/GeistMono/ttf/GeistMono-Regular.ttf"
));

pub fn run() -> iced::Result {
    let scaling = Scaling::global();
    let screen_size = scaling.screen_size;

    let window_settings = window::Settings {
        #[cfg(target_os = "linux")]
        size: screen_size,
        #[cfg(target_os = "linux")]
        position: Position::Centered,
        #[cfg(not(target_os = "linux"))]
        maximized: true,
        ..Default::default()
    };

    iced::application(Halvora::new, update, view)
        .font(GEIST_MONO_BYTES)
        .default_font(Font::with_name("Geist Mono"))
        .window(window_settings)
        .run()
}

struct Halvora;

impl Default for Halvora {
    fn default() -> Self {
        Self
    }
}

impl Halvora {
    fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    HalvingSelected(u32),
}

fn update(_state: &mut Halvora, message: Message) {
    match message {
        Message::HalvingSelected(_n) => {
            // No action yet — placeholder for future functionality
        }
    }
}

fn view(_state: &Halvora) -> Element<'_, Message> {
    row![
        halving_sidebar::view(),
        dashboard::view(),
        blockchain_sidebar::view(),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}