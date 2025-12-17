//! Settings view - App configuration

use crate::app::IkkiApp;
use crate::components::{card, divider, network_badge};
use crate::message::Message;
use crate::theme::{self, spacing, typography};
use iced::widget::{Space, button, column, container, horizontal_space, row, text, text_input};
use iced::{Alignment, Element, Length};

/// Settings view for app configuration
pub fn view(app: &IkkiApp) -> Element<Message> {
    let title = text("Settings")
        .size(typography::H2)
        .style(theme::text_style::primary());

    // Network section
    let network_name = app
        .config
        .as_ref()
        .map(|c| c.network.name())
        .unwrap_or("Not connected");

    let network_section = card(column![
        row![
            column![
                text("Network")
                    .size(typography::BODY)
                    .style(theme::text_style::primary()),
                Space::with_height(spacing::XXS),
                text("Current Zcash network")
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::secondary()),
            ],
            horizontal_space(),
            network_badge(network_name),
        ]
        .align_y(Alignment::Center),
    ]);

    // Lightwalletd section
    let lightwalletd_section = card(column![
        text("Lightwalletd Server")
            .size(typography::BODY)
            .style(theme::text_style::primary()),
        Space::with_height(spacing::XXS),
        text("Server used to sync your wallet")
            .size(typography::BODY_SMALL)
            .style(theme::text_style::secondary()),
        Space::with_height(spacing::MD),
        text_input("https://...", &app.lightwalletd_url)
            .padding(spacing::SM)
            .size(typography::BODY_SMALL)
            .style(theme::input_style::default)
            .on_input(Message::LightwalletdChanged),
    ]);

    // Sync section
    let sync_status = if app.sync_state.syncing {
        format!("Syncing... {:.0}%", app.sync_state.progress() * 100.0)
    } else {
        format!("Block {}", app.block_height)
    };

    let sync_section = card(column![
        row![
            column![
                text("Wallet Sync")
                    .size(typography::BODY)
                    .style(theme::text_style::primary()),
                Space::with_height(spacing::XXS),
                text(sync_status)
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::secondary()),
            ],
            horizontal_space(),
            button(text("Sync Now").size(typography::BODY_SMALL))
                .padding([spacing::XS, spacing::MD])
                .style(theme::button_style::secondary)
                .on_press(Message::SyncWallet),
        ]
        .align_y(Alignment::Center),
    ]);

    // About section
    let about_section = card(column![
        text("About Ikki")
            .size(typography::BODY)
            .style(theme::text_style::primary()),
        Space::with_height(spacing::MD),
        info_row("Version", "0.1.0"),
        divider(),
        info_row("Built with", "Zcash + Rust + iced"),
        divider(),
        info_row("Network", network_name.to_string()),
    ]);

    // Security section
    let security_section = card(column![
        text("Security")
            .size(typography::BODY)
            .style(theme::text_style::primary()),
        Space::with_height(spacing::MD),
        row![
            column![
                text("Seed Phrase")
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::primary()),
                Space::with_height(spacing::XXS),
                text("Your 24-word recovery phrase")
                    .size(typography::CAPTION)
                    .style(theme::text_style::secondary()),
            ],
            horizontal_space(),
            if app.show_seed {
                button(text("Hide").size(typography::BODY_SMALL))
                    .padding([spacing::XS, spacing::MD])
                    .style(theme::button_style::secondary)
                    .on_press(Message::HideSeed)
            } else {
                button(text("Show").size(typography::BODY_SMALL))
                    .padding([spacing::XS, spacing::MD])
                    .style(theme::button_style::danger)
                    .on_press(Message::ShowSeed)
            },
        ]
        .align_y(Alignment::Center),
    ]);

    // Danger zone (testnet only)
    let danger_section = container(
        column![
            row![
                text("!")
                    .size(typography::BODY)
                    .style(theme::text_style::warning()),
                Space::with_width(spacing::SM),
                text("Testnet Mode")
                    .size(typography::BODY)
                    .style(theme::text_style::warning()),
            ],
            Space::with_height(spacing::XS),
            text("This wallet is configured for testnet. Do not use mainnet funds.")
                .size(typography::BODY_SMALL)
                .style(theme::text_style::secondary()),
        ]
        .padding(spacing::MD),
    )
    .style(theme::container_style::warning_pill)
    .width(Length::Fill);

    column![
        title,
        Space::with_height(spacing::LG),
        network_section,
        Space::with_height(spacing::MD),
        lightwalletd_section,
        Space::with_height(spacing::MD),
        sync_section,
        Space::with_height(spacing::MD),
        security_section,
        Space::with_height(spacing::MD),
        about_section,
        Space::with_height(spacing::LG),
        danger_section,
    ]
    .width(Length::Fill)
    .max_width(600)
    .into()
}

fn info_row<'a>(label: &'a str, value: impl Into<String>) -> Element<'a, Message> {
    let value_str: String = value.into();
    row![
        text(label)
            .size(typography::BODY_SMALL)
            .style(theme::text_style::secondary()),
        horizontal_space(),
        text(value_str)
            .size(typography::BODY_SMALL)
            .style(theme::text_style::primary()),
    ]
    .padding([spacing::XS, 0.0])
    .into()
}
