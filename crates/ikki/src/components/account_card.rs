//! Account card component - Displays wallet balance in a card layout

use crate::message::Message;
use crate::theme::{
    self, colors, format_zec_display, radius, spacing, truncate_address, typography,
};
use iced::widget::{Space, column, container, horizontal_space, row, text};
use iced::{Alignment, Element, Length};

/// Creates a beautiful account card displaying the wallet balance
///
/// Inspired by Revolut's card design with a gradient background,
/// prominent balance display, and subtle details.
pub fn account_card<'a>(
    balance: u64,
    address: &'a str,
    network: &'a str,
    syncing: bool,
) -> Element<'a, Message> {
    let (whole, decimal) = format_zec_display(balance);

    // Network indicator
    let network_pill = {
        let is_testnet = network.to_lowercase().contains("test");
        let pill_bg = if is_testnet {
            iced::Color::from_rgba(1.0, 1.0, 1.0, 0.15)
        } else {
            iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2)
        };

        container(
            text(network.to_uppercase())
                .size(typography::OVERLINE)
                .style(theme::text_style::on_dark()),
        )
        .padding([spacing::XXXS, spacing::XS])
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(pill_bg)),
            border: iced::Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    };

    // Sync indicator
    let sync_indicator = if syncing {
        container(
            row![
                text("...")
                    .size(typography::CAPTION)
                    .style(theme::text_style::on_dark()),
                Space::with_width(spacing::XXS),
                text("Syncing")
                    .size(typography::CAPTION)
                    .style(theme::text_style::on_dark()),
            ]
            .align_y(Alignment::Center),
        )
        .padding([spacing::XXXS, spacing::XS])
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                1.0, 1.0, 1.0, 0.15,
            ))),
            border: iced::Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
    } else {
        container(Space::with_height(0))
    };

    // Top row with network and sync
    let top_row =
        row![network_pill, horizontal_space(), sync_indicator,].align_y(Alignment::Center);

    // Balance label
    let balance_label = text("Shielded Balance")
        .size(typography::BODY_SMALL)
        .style(|_| text::Style {
            color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
        });

    // Main balance display - large and prominent
    let balance_display = row![
        text(whole)
            .size(typography::DISPLAY)
            .style(theme::text_style::on_dark()),
        text(decimal).size(typography::H2).style(|_| text::Style {
            color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
        }),
        Space::with_width(spacing::XS),
        text("ZEC").size(typography::H3).style(|_| text::Style {
            color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
        }),
    ]
    .align_y(Alignment::End);

    // Address at bottom (truncated)
    let address_display = row![
        text("*").size(typography::CAPTION).style(|_| text::Style {
            color: Some(colors::SHIELDED),
        }),
        Space::with_width(spacing::XXS),
        text(truncate_address(address, 8))
            .size(typography::CAPTION)
            .style(|_| text::Style {
                color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
            }),
    ]
    .align_y(Alignment::Center);

    // Card content
    let content = column![
        top_row,
        Space::with_height(spacing::XL),
        balance_label,
        Space::with_height(spacing::XXS),
        balance_display,
        Space::with_height(spacing::LG),
        address_display,
    ]
    .padding(spacing::LG);

    // Card with gradient background
    container(content)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(colors::CARD_GRADIENT_START)),
            border: iced::Border {
                radius: radius::XL.into(),
                ..Default::default()
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 12.0),
                blur_radius: 40.0,
            },
            ..Default::default()
        })
        .into()
}

/// Creates a smaller account card for the card list/carousel
#[allow(dead_code)]
pub fn account_card_mini<'a>(
    balance: u64,
    network: &'a str,
    selected: bool,
) -> Element<'a, Message> {
    let (whole, decimal) = format_zec_display(balance);

    let bg_color = if selected {
        colors::CARD_GRADIENT_START
    } else {
        colors::CARD_GRADIENT_END
    };

    let opacity = if selected { 1.0 } else { 0.7 };

    let content = column![
        text(network.to_uppercase())
            .size(typography::OVERLINE)
            .style(move |_| text::Style {
                color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.6 * opacity)),
            }),
        Space::with_height(spacing::SM),
        row![
            text(whole)
                .size(typography::H3)
                .style(move |_| text::Style {
                    color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, opacity)),
                }),
            text(decimal)
                .size(typography::BODY_SMALL)
                .style(move |_| text::Style {
                    color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.6 * opacity)),
                }),
            Space::with_width(spacing::XXS),
            text("ZEC")
                .size(typography::BODY_SMALL)
                .style(move |_| text::Style {
                    color: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.5 * opacity)),
                }),
        ]
        .align_y(Alignment::End),
    ]
    .padding(spacing::MD);

    container(content)
        .width(180)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                radius: radius::LG.into(),
                width: if selected { 2.0 } else { 0.0 },
                color: if selected {
                    colors::ACCENT
                } else {
                    iced::Color::TRANSPARENT
                },
            },
            shadow: if selected {
                iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 16.0,
                }
            } else {
                iced::Shadow::default()
            },
            ..Default::default()
        })
        .into()
}
