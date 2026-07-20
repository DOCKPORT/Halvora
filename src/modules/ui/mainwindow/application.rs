use iced::widget::row;
use iced::{Element, Subscription, window, Font, Length};
use iced::window::Position;
use rusqlite::Connection;
use std::path::PathBuf;
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
        .subscription(subscription)
        .run()
}

struct Halvora {
    selected_halving: Option<u32>,
    current_tip_height: u32,
}

impl Halvora {
    fn new() -> Self {
        let current_tip_height = Self::load_tip_height();
        Self {
            selected_halving: None,
            current_tip_height,
        }
    }

    /// Query the most recent tip height from the database.
    fn load_tip_height() -> u32 {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_path = base.join("Halvora").join("Mempool").join("blocks.db");
        if let Ok(conn) = Connection::open(&db_path) {
            if let Ok(height) = conn.query_row(
                "SELECT height FROM current_tip LIMIT 1",
                [],
                |row| row.get(0),
            ) {
                return height;
            }
        }
        0
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    HalvingSelected(u32),
    Tick,
}

fn subscription(_state: &Halvora) -> Subscription<Message> {
    iced::time::every(std::time::Duration::from_secs(600)).map(|_| Message::Tick)
}

fn update(state: &mut Halvora, message: Message) {
    match message {
        Message::HalvingSelected(n) => {
            state.selected_halving = Some(n);
        }
        Message::Tick => {
            crate::modules::api::mempool::rest::halve_blocks::fetch_and_store();
            state.current_tip_height = Halvora::load_tip_height();
        }
    }
}

fn view(state: &Halvora) -> Element<'_, Message> {
    row![
        halving_sidebar::view(state.selected_halving),
        dashboard::view(state.selected_halving),
        blockchain_sidebar::view(state.current_tip_height),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}