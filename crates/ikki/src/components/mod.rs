//! Reusable UI components for Ikki

mod account_card;
mod action_button;
mod transaction_item;

pub use account_card::account_card;
pub use action_button::{ActionType, action_button};
pub use transaction_item::transaction_item;

use crate::message::Message;
use crate::theme::{self, colors, spacing, typography};
use iced::widget::{Space, button, column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

/// Section header with optional action
pub fn section_header<'a>(
    title: &'a str,
    action: Option<(&'a str, Message)>,
) -> Element<'a, Message> {
    let title_text = text(title)
        .size(typography::H3)
        .style(theme::text_style::primary());

    let content = if let Some((action_label, msg)) = action {
        row![
            title_text,
            horizontal_space(),
            button(
                text(action_label)
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::brand())
            )
            .padding([spacing::XXS, spacing::XS])
            .style(theme::button_style::ghost)
            .on_press(msg),
        ]
        .align_y(Alignment::Center)
    } else {
        row![title_text,]
    };

    content.width(Length::Fill).into()
}

/// Empty state placeholder
pub fn empty_state<'a>(icon: &'a str, title: &'a str, subtitle: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(icon).size(48.0),
            Space::with_height(spacing::MD),
            text(title)
                .size(typography::BODY_LARGE)
                .style(theme::text_style::primary()),
            Space::with_height(spacing::XXS),
            text(subtitle)
                .size(typography::BODY_SMALL)
                .style(theme::text_style::secondary()),
        ]
        .align_x(Alignment::Center)
        .padding(spacing::XXL),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}

/// Loading spinner
pub fn loading_spinner<'a>(message: &'a str) -> Element<'a, Message> {
    container(
        column![
            text("...")
                .size(typography::H2)
                .style(theme::text_style::secondary()),
            Space::with_height(spacing::SM),
            text(message)
                .size(typography::BODY_SMALL)
                .style(theme::text_style::secondary()),
        ]
        .align_x(Alignment::Center)
        .padding(spacing::XL),
    )
    .width(Length::Fill)
    .center_x(Length::Fill)
    .into()
}

/// Status pill component
pub fn status_pill<'a>(label: &'a str, success: bool) -> Element<'a, Message> {
    let style = if success {
        theme::container_style::success_pill
    } else {
        theme::container_style::warning_pill
    };

    let text_color = if success {
        colors::SUCCESS
    } else {
        colors::WARNING
    };

    container(
        text(label)
            .size(typography::CAPTION)
            .style(move |_| text::Style {
                color: Some(text_color),
            }),
    )
    .padding([spacing::XXS, spacing::XS])
    .style(style)
    .into()
}

/// Divider line
pub fn divider<'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .height(1)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(colors::DIVIDER)),
            ..Default::default()
        })
        .into()
}

/// Info row (label: value)
pub fn info_row<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(label)
            .size(typography::BODY_SMALL)
            .style(theme::text_style::secondary())
            .width(Length::FillPortion(1)),
        text(value)
            .size(typography::BODY_SMALL)
            .style(theme::text_style::primary())
            .width(Length::FillPortion(2)),
    ]
    .spacing(spacing::SM)
    .padding([spacing::XS, 0.0])
    .into()
}

/// Sync progress indicator
pub fn sync_indicator<'a>(syncing: bool, progress: f32, block_height: u64) -> Element<'a, Message> {
    let status_text = if syncing {
        format!("Syncing... {:.0}%", progress * 100.0)
    } else {
        format!("Block {block_height}")
    };

    let indicator = if syncing {
        text("...").style(theme::text_style::warning())
    } else {
        text("*").style(theme::text_style::success())
    };

    container(
        row![
            indicator.size(typography::CAPTION),
            Space::with_width(spacing::XXS),
            text(status_text)
                .size(typography::CAPTION)
                .style(theme::text_style::secondary()),
        ]
        .align_y(Alignment::Center),
    )
    .padding([spacing::XXS, spacing::XS])
    .style(theme::container_style::pill)
    .into()
}

/// Network badge
pub fn network_badge<'a>(network: &'a str) -> Element<'a, Message> {
    let is_testnet = network.to_lowercase().contains("test");
    let style = if is_testnet {
        theme::container_style::warning_pill
    } else {
        theme::container_style::success_pill
    };

    let text_color = if is_testnet {
        colors::WARNING
    } else {
        colors::SUCCESS
    };

    container(
        text(network)
            .size(typography::OVERLINE)
            .style(move |_| text::Style {
                color: Some(text_color),
            }),
    )
    .padding([spacing::XXXS, spacing::XS])
    .style(style)
    .into()
}

/// Card container with consistent styling
pub fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(spacing::LG)
        .style(theme::container_style::card)
        .width(Length::Fill)
        .into()
}

/// Elevated card container
pub fn card_elevated<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(spacing::LG)
        .style(theme::container_style::card_elevated)
        .width(Length::Fill)
        .into()
}
