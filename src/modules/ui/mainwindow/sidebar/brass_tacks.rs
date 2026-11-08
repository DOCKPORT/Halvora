use crate::modules::ui::mainwindow::application::Message;
use crate::modules::ui::scaling::sp;
use crate::modules::ui::theme;
use iced::widget::{button, text};
use iced::{Element, Length, border};

/// The semibold monospaced font used for the button label.
const FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("Geist Mono"),
    weight: iced::font::Weight::Semibold,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

/// Neutral button style for the Brass Tacks entry.
///
/// Brass Tacks is not a period, so it never uses the red/green P/L tint. It
/// reuses the grey halving button fill. Only the hover state changes the
/// fill; the button has no persistent selected border.
fn button_style(status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => theme::HALVING_BUTTON_HOVER,
        _ => theme::HALVING_BUTTON_BACKGROUND,
    };

    let border = border::rounded(8)
        .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
        .width(1.0);

    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color: theme::HALVING_BUTTON_TEXT,
        border,
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

/// Render the Brass Tacks button.
///
/// The button matches the Year-Over-Year button size and font, but uses the
/// neutral grey fill. A press emits `Message::BrassTacksSelected`.
pub fn view<'a>() -> Element<'a, Message> {
    button(
        text("Brass-Tacks")
            .size(sp(16.0))
            .width(Length::Shrink)
            .center()
            .font(FONT),
    )
    .width(Length::Fill)
    .height(Length::Fixed(sp(36.0)))
    .padding(0)
    .on_press(Message::BrassTacksSelected)
    .style(move |_theme, status| button_style(status))
    .into()
}
