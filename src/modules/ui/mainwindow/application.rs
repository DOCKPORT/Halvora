use iced::widget::{container, row, rule};
use iced::{Element, window, Length};
use iced::window::Position;
use crate::modules::ui::theme;
use crate::modules::ui::scaling::Scaling;
use crate::modules::ui::mainwindow::sidebar::halving_sidebar;
use crate::modules::ui::mainwindow::sidebar::blockchain_sidebar;

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
pub enum Message {}

fn update(_state: &mut Halvora, _message: Message) {}

fn view(_state: &Halvora) -> Element<'_, Message> {
    // Orange border separator between sidebars and content
    let border = |_theme: &iced::Theme| -> rule::Style {
        rule::Style {
            color: theme::SIDEBAR_BORDER_COLOR,
            fill_mode: rule::FillMode::Full,
            radius: iced::border::Radius::default(),
            snap: false,
        }
    };

    row![
        halving_sidebar::view(),
        rule::vertical(2).style(border),
        container(iced::widget::column![])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| {
                container::Style::default().background(
                    iced::Background::Color(theme::MAINWINDOW_BACKGROUND)
                )
            }),
        rule::vertical(2).style(border),
        blockchain_sidebar::view(),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}