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

struct Halvora {
    selected_halving: Option<u32>,
}

impl Default for Halvora {
    fn default() -> Self {
        Self {
            selected_halving: None,
        }
    }
}

impl Halvora {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    HalvingSelected(u32),
}

fn update(state: &mut Halvora, message: Message) {
    match message {
        Message::HalvingSelected(n) => {
            state.selected_halving = Some(n);
        }
    }
}

fn view(state: &Halvora) -> Element<'_, Message> {
    row![
        halving_sidebar::view(state.selected_halving),
        dashboard::view(state.selected_halving),
        blockchain_sidebar::view(),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}