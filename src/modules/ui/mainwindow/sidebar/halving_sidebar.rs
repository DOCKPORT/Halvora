use iced::widget::{button, container, image, scrollable, Column, Row};
use iced::{border, ContentFit, Element, Length};
use crate::modules::ui::theme;

pub fn view<'a>(selected_halving: Option<u32>, yoy_selected: bool) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;

    // Build 32 buttons in a 2-column grid: 16 rows of [H-n, H-(n+1)]
    let mut rows: Vec<Element<'a, Message>> = Vec::with_capacity(16);
    for i in (1..=32).step_by(2) {
        let row = Row::with_children(vec![
            halving_button(i, selected_halving),
            if i + 1 <= 32 {
                halving_button(i + 1, selected_halving)
            } else {
                container(iced::widget::column![])
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(36.0))
                    .into()
            },
        ])
        .spacing(8)
        .padding(iced::Padding::new(0.0).left(21.0).right(21.0))
        .width(Length::Fill)
        .into();
        rows.push(row);
    }

    let content = Column::with_children({
        let mut children: Vec<Element<'a, Message>> = Vec::with_capacity(17);

        // Logo at top
        children.push(
            image("Halvora_Logo/Halvora.png")
                .content_fit(ContentFit::Contain)
                .width(Length::Fill)
                .height(Length::Fixed(80.0))
                .into(),
        );

        // Spacer
        children.push(iced::widget::space().height(Length::Fixed(8.0)).into());

        // YoY button — same padding & width as grid rows for centering
        children.push(
            Row::with_children(vec![yoy_button(yoy_selected)])
                .padding(iced::Padding::new(0.0).left(21.0).right(21.0))
                .width(Length::Fill)
                .into(),
        );

        // Spacer before grid
        children.push(iced::widget::space().height(Length::Fixed(8.0)).into());

        // Grid rows
        for row in rows {
            children.push(row);
        }

        children
    })
    .spacing(8)
    .padding(0);

    container(scrollable(content))
        .width(Length::Fixed(250.0))
        .height(Length::Fill)
        .padding(0)
        .style(|_theme| {
            container::Style::default().background(
                iced::Background::Color(theme::SIDEBAR_BACKGROUND)
            )
        })
        .into()
}

fn yoy_button<'a>(
    is_selected: bool,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;

    button(
        iced::widget::text("Year-Over-Year")
            .size(16)
            .width(Length::Shrink)
            .center()
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            }),
    )
    .width(Length::Fill)
    .height(Length::Fixed(36.0))
    .padding(0)
    .on_press(Message::YoYSelected)
    .style(move |_theme, status| {
        let background = match status {
            button::Status::Hovered => theme::HALVING_BUTTON_HOVER,
            _ => theme::HALVING_BUTTON_BACKGROUND,
        };

        let text_color = theme::HALVING_BUTTON_TEXT;

        let border = if is_selected {
            border::rounded(8).color(theme::HALVING_BUTTON_TEXT).width(1.5)
        } else {
            border::rounded(8).color(iced::Color::TRANSPARENT).width(0)
        };

        button::Style {
            background: Some(iced::Background::Color(background)),
            text_color,
            border,
            shadow: iced::Shadow::default(),
            snap: false,
        }
    })
    .into()
}

fn halving_button<'a>(
    num: u32,
    selected_halving: Option<u32>,
) -> Element<'a, crate::modules::ui::mainwindow::application::Message> {
    use crate::modules::ui::mainwindow::application::Message;
    let is_selected = selected_halving == Some(num);

    button(
        iced::widget::text(format!("H-{}", num))
            .size(16)
            .width(Length::Shrink)
            .center()
            .font(iced::Font {
                family: iced::font::Family::Name("Geist Mono"),
                weight: iced::font::Weight::Semibold,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            }),
    )
    .width(Length::Fixed(100.0))
    .height(Length::Fixed(36.0))
    .padding(0)
    .on_press(Message::HalvingSelected(num))
    .style(move |_theme, status| {
        let background = match status {
            button::Status::Hovered => theme::HALVING_BUTTON_HOVER,
            _ => theme::HALVING_BUTTON_BACKGROUND,
        };

        let text_color = theme::HALVING_BUTTON_TEXT;

        let border = if is_selected {
            border::rounded(8).color(theme::HALVING_BUTTON_TEXT).width(1.5)
        } else {
            border::rounded(8).color(iced::Color::TRANSPARENT).width(0)
        };

        button::Style {
            background: Some(iced::Background::Color(background)),
            text_color,
            border,
            shadow: iced::Shadow::default(),
            snap: false,
        }
    })
    .into()
}
