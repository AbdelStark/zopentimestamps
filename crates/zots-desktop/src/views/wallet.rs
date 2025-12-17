//! Wallet view - Wallet management

use crate::app::ZotsApp;
use crate::message::Message;
use crate::theme::{self, colors};
use iced::widget::{button, column, container, horizontal_space, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = row![
        text(">").size(28),
        Space::with_width(12),
        text("Wallet").size(24),
    ]
    .align_y(Alignment::Center);

    let description = text("Manage your Zcash testnet wallet for timestamping")
        .size(14)
        .style(theme::text_style::muted());

    // Security warning
    let warning = container(
        row![
            text("!").size(20),
            Space::with_width(12),
            column![
                text("Security Warning")
                    .size(14)
                    .style(theme::text_style::warning()),
                Space::with_height(4),
                text("Never enter your mainnet seed phrase here. This app is for testnet only. Your seed is stored in the ZOTS_SEED environment variable.")
                    .size(12)
                    .style(theme::text_style::muted()),
            ],
        ]
        .align_y(Alignment::Start)
        .padding(16),
    )
    .style(|theme| {
        let mut style = theme::container_style::card(theme);
        style.border.color = colors::WARNING;
        style
    })
    .width(Length::Fill);

    // Wallet status
    let wallet_content = if app.config.is_some() {
        // Wallet is configured
        let balance_zec = app.balance as f64 / 100_000_000.0;

        let sync_btn = if app.wallet_syncing {
            button(
                row![
                    text(app.spinner()).size(14),
                    Space::with_width(8),
                    text("Syncing...").size(14),
                ]
                .align_y(Alignment::Center),
            )
            .padding([12, 20])
            .style(theme::button_style::secondary)
        } else {
            button(
                row![
                    text(">").size(14),
                    Space::with_width(8),
                    text("Sync Now").size(14),
                ]
                .align_y(Alignment::Center),
            )
            .padding([12, 20])
            .style(theme::button_style::secondary)
            .on_press(Message::SyncWallet)
        };

        let wallet_col = column![
            row![
                text("✓").size(18).color(colors::SUCCESS),
                Space::with_width(12),
                text("Wallet Configured").size(16).color(colors::SUCCESS),
            ]
            .align_y(Alignment::Center),
            Space::with_height(24),
            // Balance card
            container(
                column![
                    text("Balance").size(12).style(theme::text_style::muted()),
                    Space::with_height(4),
                    text(format!("{balance_zec:.8} ZEC"))
                        .size(28)
                        .style(theme::text_style::accent()),
                ]
                .padding(20)
                .align_x(Alignment::Center),
            )
            .style(theme::container_style::surface)
            .width(Length::Fill),
            Space::with_height(16),
            // Info rows
            info_row(
                "Network",
                app.config
                    .as_ref()
                    .map(|c| c.network.name())
                    .unwrap_or("Unknown")
                    .to_string(),
            ),
            info_row("Block Height", app.block_height.to_string()),
            info_row(
                "Status",
                if app.wallet_syncing {
                    "Syncing...".to_string()
                } else {
                    "Ready".to_string()
                },
            ),
            Space::with_height(20),
            row![sync_btn, horizontal_space(),],
        ]
        .padding(24);

        // Add error display if present
        let wallet_col = if let Some(error) = &app.wallet_error {
            wallet_col.push(
                container(
                    row![
                        text("!").size(14).color(colors::ERROR),
                        Space::with_width(8),
                        text(error).size(12).color(colors::ERROR),
                    ]
                    .align_y(Alignment::Center)
                    .padding([12, 0]),
                ),
            )
        } else {
            wallet_col
        };

        container(wallet_col)
            .style(theme::container_style::card)
            .width(Length::Fill)
    } else {
        // No wallet configured - show seed input
        let setup_col = column![
            row![
                text(">").size(18),
                Space::with_width(12),
                text("Configure Wallet").size(16),
            ]
            .align_y(Alignment::Center),
            Space::with_height(16),
            text("Enter your BIP-39 seed phrase (24 words) to configure the wallet.")
                .size(13)
                .style(theme::text_style::muted()),
            Space::with_height(16),
            text("Seed Phrase").size(13),
            Space::with_height(8),
            text_input("Enter your 24-word seed phrase...", &app.seed_input,)
                .padding(12)
                .size(14)
                .style(theme::input_style::default)
                .on_input(Message::SeedInputChanged)
                .secure(true),
            Space::with_height(16),
            button(
                row![
                    text(">").size(14),
                    Space::with_width(8),
                    text("Save & Connect").size(14),
                ]
                .align_y(Alignment::Center),
            )
            .padding([12, 20])
            .style(theme::button_style::primary)
            .on_press(Message::SaveSeed),
        ]
        .padding(24);

        // Add error display if present
        let setup_col = if let Some(error) = &app.wallet_error {
            setup_col.push(container(
                row![
                    text("!").size(14).color(colors::ERROR),
                    Space::with_width(8),
                    text(error).size(12).color(colors::ERROR),
                ]
                .align_y(Alignment::Center)
                .padding([12, 0]),
            ))
        } else {
            setup_col
        };

        container(setup_col)
            .style(theme::container_style::card)
            .width(Length::Fill)
    };

    // Environment variable hint
    let env_hint = container(
        column![
            text("※ Tip: Set ZOTS_SEED environment variable")
                .size(13)
                .style(theme::text_style::muted()),
            Space::with_height(8),
            text("You can set your seed phrase in a .env file or as an environment variable to avoid entering it each time.")
                .size(12)
                .style(theme::text_style::dim()),
            Space::with_height(8),
            container(
                text("export ZOTS_SEED=\"your seed phrase here\"")
                    .size(12)
                    .style(theme::text_style::accent()),
            )
            .style(theme::container_style::surface)
            .padding(12),
        ]
        .padding(16),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    column![
        title,
        Space::with_height(8),
        description,
        Space::with_height(24),
        warning,
        Space::with_height(24),
        wallet_content,
        Space::with_height(24),
        env_hint,
    ]
    .width(Length::Fill)
    .into()
}

fn info_row(label: &'static str, value: String) -> Element<'static, Message> {
    container(
        row![
            text(format!("{label}:"))
                .size(13)
                .style(theme::text_style::muted())
                .width(120),
            text(value).size(13),
        ]
        .align_y(Alignment::Center)
        .padding([6, 0]),
    )
    .into()
}
