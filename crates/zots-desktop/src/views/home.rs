//! Home view - Dashboard overview

use crate::app::ZotsApp;
use crate::message::{Message, View};
use crate::theme::{self, colors};
use iced::widget::{button, column, container, horizontal_space, row, text, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = text("Welcome to zOpenTimestamps")
        .size(28)
        .style(theme::text_style::primary());

    let subtitle = text("Timestamp files on the Zcash blockchain with privacy-preserving shielded transactions")
        .size(14)
        .style(theme::text_style::muted());

    // Quick stats
    let balance_zec = app.balance as f64 / 100_000_000.0;
    let balance_str = format!("{balance_zec:.8} ZEC");
    let block_str = app.block_height.to_string();
    let network_str = app
        .config
        .as_ref()
        .map(|c| c.network.name())
        .unwrap_or("Not configured")
        .to_string();
    let stats = container(
        row![
            stat_card("💰", "Balance", balance_str),
            Space::with_width(16),
            stat_card("📦", "Block Height", block_str),
            Space::with_width(16),
            stat_card("🔗", "Network", network_str),
        ]
        .padding([0, 0]),
    );

    // Quick actions
    let actions_title = text("Quick Actions")
        .size(18)
        .style(theme::text_style::muted());

    let stamp_action = action_card(
        "📝",
        "Create Timestamp",
        "Timestamp a file or hash on the Zcash blockchain",
        View::Stamp,
    );

    let verify_action = action_card(
        "🔍",
        "Verify Proof",
        "Verify a timestamp proof against the blockchain",
        View::Verify,
    );

    let wallet_action = action_card(
        "💰",
        "Wallet",
        "View balance and configure your wallet",
        View::Wallet,
    );

    let actions = row![stamp_action, Space::with_width(16), verify_action, Space::with_width(16), wallet_action,];

    // Security warning
    let warning = container(
        row![
            text("⚠️").size(20),
            Space::with_width(12),
            column![
                text("Testnet Only")
                    .size(14)
                    .style(theme::text_style::warning()),
                text("This is experimental software. Do not use on mainnet with real funds.")
                    .size(12)
                    .style(theme::text_style::muted()),
            ],
        ]
        .align_y(Alignment::Center)
        .padding(16),
    )
    .style(|theme| {
        let mut style = theme::container_style::card(theme);
        style.border.color = colors::WARNING;
        style
    })
    .width(Length::Fill);

    // About section
    let about = container(
        column![
            text("About zOpenTimestamps").size(16),
            Space::with_height(8),
            text("zOpenTimestamps is a Zcash blockchain timestamping tool inspired by OpenTimestamps. It creates cryptographic proofs that data existed at a specific point in time using Zcash's shielded transactions.")
                .size(13)
                .style(theme::text_style::muted()),
            Space::with_height(16),
            row![
                text("Version 0.1.1").size(12).style(theme::text_style::dim()),
                horizontal_space(),
                text("MIT License").size(12).style(theme::text_style::dim()),
            ],
        ]
        .padding(16),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    container(
        column![
            title,
            Space::with_height(8),
            subtitle,
            Space::with_height(32),
            stats,
            Space::with_height(32),
            actions_title,
            Space::with_height(16),
            actions,
            Space::with_height(32),
            warning,
            Space::with_height(24),
            about,
        ]
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn stat_card(icon: &'static str, label: &'static str, value: String) -> Element<'static, Message> {
    container(
        column![
            row![
                text(icon).size(24),
                Space::with_width(8),
                text(label).size(12).style(theme::text_style::muted()),
            ]
            .align_y(Alignment::Center),
            Space::with_height(8),
            text(value).size(18).style(theme::text_style::accent()),
        ]
        .padding(16),
    )
    .style(theme::container_style::card)
    .width(Length::FillPortion(1))
    .into()
}

fn action_card<'a>(
    icon: &'a str,
    title: &'a str,
    description: &'a str,
    target: View,
) -> Element<'a, Message> {
    button(
        column![
            text(icon).size(32),
            Space::with_height(12),
            text(title).size(16),
            Space::with_height(4),
            text(description)
                .size(12)
                .style(theme::text_style::muted()),
        ]
        .padding(20)
        .align_x(Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .style(theme::button_style::secondary)
    .on_press(Message::NavigateTo(target))
    .into()
}
