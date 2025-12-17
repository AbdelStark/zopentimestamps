//! Receive view - Display address and QR code

use crate::app::IkkiApp;
use crate::message::Message;
use crate::theme::{self, colors, radius, spacing, typography};
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

/// Receive view showing the wallet address and QR code
///
/// Clean, centered design with:
/// - Large QR code for easy scanning
/// - Full address display with copy button
/// - Shield indicator for privacy
pub fn view(app: &IkkiApp) -> Element<Message> {
    let title = text("Receive ZEC")
        .size(typography::H2)
        .style(theme::text_style::primary());

    let subtitle = text("Share your address to receive shielded ZEC")
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    // QR code placeholder - we'll show the address prominently instead
    // Full QR code implementation would require storing Data in app state
    let qr_placeholder: Element<Message> = if !app.address.is_empty() {
        container(
            column![
                text("QR")
                    .size(typography::DISPLAY)
                    .style(theme::text_style::secondary()),
                Space::with_height(spacing::SM),
                text("Scan to receive")
                    .size(typography::CAPTION)
                    .style(theme::text_style::tertiary()),
            ]
            .align_x(Alignment::Center),
        )
        .width(200)
        .height(200)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::WHITE)),
            border: iced::Border {
                radius: radius::LG.into(),
                color: colors::BORDER,
                width: 1.0,
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
    } else {
        container(
            column![
                text("...")
                    .size(typography::H1)
                    .style(theme::text_style::secondary()),
                Space::with_height(spacing::SM),
                text("Loading address...")
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::secondary()),
            ]
            .align_x(Alignment::Center),
        )
        .padding(spacing::XXL)
        .into()
    };

    // Address type indicator
    let address_type = container(
        row![
            text("*")
                .size(typography::CAPTION)
                .style(theme::text_style::shielded()),
            Space::with_width(spacing::XXS),
            text("Shielded Address")
                .size(typography::CAPTION)
                .style(theme::text_style::shielded()),
        ]
        .align_y(Alignment::Center),
    )
    .padding([spacing::XXS, spacing::SM])
    .style(|_| container::Style {
        background: Some(iced::Background::Color(iced::Color {
            a: 0.1,
            ..colors::SHIELDED
        })),
        border: iced::Border {
            radius: radius::FULL.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Full address display - use owned String to avoid lifetime issues
    let address_text = if !app.address.is_empty() {
        truncate_address(&app.address, 16)
    } else {
        "...".to_string()
    };

    let address_display = container(
        column![
            text(address_text)
                .size(typography::BODY_SMALL)
                .style(theme::text_style::primary()),
        ]
        .align_x(Alignment::Center),
    )
    .padding(spacing::MD)
    .width(Length::Fill)
    .style(theme::container_style::surface);

    // Copy button
    let copy_btn =
        button(row![text("Copy Address").size(typography::BODY),].align_y(Alignment::Center))
            .padding([spacing::MD, spacing::XL])
            .width(Length::Fill)
            .style(theme::button_style::primary)
            .on_press(Message::CopyToClipboard(app.address.clone()));

    // Share info
    let share_info = text("Only share this address with people you trust")
        .size(typography::CAPTION)
        .style(theme::text_style::tertiary());

    // Main layout
    let content = column![
        title,
        Space::with_height(spacing::XXS),
        subtitle,
        Space::with_height(spacing::XL),
        container(qr_placeholder)
            .width(Length::Fill)
            .center_x(Length::Fill),
        Space::with_height(spacing::LG),
        container(address_type)
            .width(Length::Fill)
            .center_x(Length::Fill),
        Space::with_height(spacing::MD),
        address_display,
        Space::with_height(spacing::LG),
        copy_btn,
        Space::with_height(spacing::SM),
        container(share_info).center_x(Length::Fill),
    ]
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .max_width(400);

    container(content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

/// Truncate address for display (owned version)
fn truncate_address(address: &str, chars: usize) -> String {
    if address.len() <= chars * 2 + 3 {
        address.to_string()
    } else {
        format!(
            "{}...{}",
            &address[..chars],
            &address[address.len() - chars..]
        )
    }
}
