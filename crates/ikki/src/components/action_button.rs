//! Action button component - Circular buttons for primary actions

use crate::message::Message;
use crate::theme::{self, colors, radius, spacing, typography};
use iced::widget::{Space, button, column, container, text};
use iced::{Alignment, Element, Length};

/// Action button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Send,
    Receive,
    Swap,
    More,
}

impl ActionType {
    pub fn icon(&self) -> &'static str {
        match self {
            ActionType::Send => "^",    // Arrow up
            ActionType::Receive => "v", // Arrow down
            ActionType::Swap => "<>",
            ActionType::More => "...",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActionType::Send => "Send",
            ActionType::Receive => "Receive",
            ActionType::Swap => "Swap",
            ActionType::More => "More",
        }
    }
}

/// Creates a circular action button with icon and label
///
/// Inspired by Revolut's action buttons - clean circular design
/// with subtle color coding for different actions.
pub fn action_button<'a>(action: ActionType, on_press: Message) -> Element<'a, Message> {
    let (bg_color, text_color) = match action {
        ActionType::Send => (colors::SEND, colors::TEXT_ON_DARK),
        ActionType::Receive => (colors::RECEIVE, colors::TEXT_ON_DARK),
        ActionType::Swap => (colors::INFO, colors::TEXT_ON_DARK),
        ActionType::More => (colors::SURFACE_SECONDARY, colors::TEXT_PRIMARY),
    };

    let icon_btn = button(
        container(
            text(action.icon())
                .size(typography::BODY_LARGE)
                .style(move |_| text::Style {
                    color: Some(text_color),
                }),
        )
        .width(56)
        .height(56)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .style(move |_theme, status| {
        let alpha = match status {
            button::Status::Active => 1.0,
            button::Status::Hovered => 0.9,
            button::Status::Pressed => 0.8,
            button::Status::Disabled => 0.4,
        };

        button::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: alpha,
                ..bg_color
            })),
            text_color,
            border: iced::Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            shadow: match action {
                ActionType::Send | ActionType::Receive => iced::Shadow {
                    color: iced::Color {
                        a: 0.25,
                        ..bg_color
                    },
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
                _ => iced::Shadow::default(),
            },
        }
    })
    .on_press(on_press);

    let label = text(action.label())
        .size(typography::CAPTION)
        .style(theme::text_style::secondary());

    column![icon_btn, Space::with_height(spacing::XS), label,]
        .align_x(Alignment::Center)
        .into()
}

/// Creates a row of action buttons (Send, Receive, etc.)
#[allow(dead_code)]
pub fn action_buttons<'a>(
    on_send: Message,
    on_receive: Message,
    send_disabled: bool,
) -> Element<'a, Message> {
    let send_btn = if send_disabled {
        action_button_disabled(ActionType::Send)
    } else {
        action_button(ActionType::Send, on_send)
    };

    let receive_btn = action_button(ActionType::Receive, on_receive);

    iced::widget::row![send_btn, Space::with_width(spacing::XL), receive_btn,]
        .align_y(Alignment::Start)
        .into()
}

/// Disabled action button
#[allow(dead_code)]
fn action_button_disabled<'a>(action: ActionType) -> Element<'a, Message> {
    let text_color = colors::TEXT_TERTIARY;

    let icon_btn = button(
        container(
            text(action.icon())
                .size(typography::BODY_LARGE)
                .style(move |_| text::Style {
                    color: Some(text_color),
                }),
        )
        .width(56)
        .height(56)
        .center_x(Length::Fill)
        .center_y(Length::Fill),
    )
    .style(move |_theme, _status| button::Style {
        background: Some(iced::Background::Color(colors::SURFACE_SECONDARY)),
        text_color,
        border: iced::Border {
            radius: radius::FULL.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let label = text(action.label())
        .size(typography::CAPTION)
        .style(theme::text_style::tertiary());

    column![icon_btn, Space::with_height(spacing::XS), label,]
        .align_x(Alignment::Center)
        .into()
}
