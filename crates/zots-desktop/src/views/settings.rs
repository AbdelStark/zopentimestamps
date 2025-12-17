//! Settings view - Application settings

use crate::app::ZotsApp;
use crate::message::Message;
use crate::theme;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = row![
        text(">").size(28),
        Space::with_width(12),
        text("Settings").size(24),
    ]
    .align_y(Alignment::Center);

    let description = text("Configure application settings")
        .size(14)
        .style(theme::text_style::muted());

    // Explorer URL setting
    let explorer_section = container(
        column![
            text("Block Explorer").size(16),
            Space::with_height(8),
            text("URL for viewing Zcash transactions")
                .size(12)
                .style(theme::text_style::dim()),
            Space::with_height(12),
            text_input("https://blockexplorer.one/zcash/testnet", &app.explorer_url)
                .padding(12)
                .size(14)
                .style(theme::input_style::default)
                .on_input(Message::ExplorerUrlChanged),
        ]
        .padding(20),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    // Lightwalletd URL setting
    let lightwalletd_section = container(
        column![
            text("Lightwalletd Server").size(16),
            Space::with_height(8),
            text("gRPC endpoint for Zcash light client")
                .size(12)
                .style(theme::text_style::dim()),
            Space::with_height(12),
            text_input(
                "https://zcash.mysideoftheweb.com:19067",
                &app.lightwalletd_url,
            )
            .padding(12)
            .size(14)
            .style(theme::input_style::default)
            .on_input(Message::LightwalletdUrlChanged),
        ]
        .padding(20),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    // Action buttons
    let save_btn = button(
        row![
            text(">").size(14),
            Space::with_width(8),
            text("Save Settings").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 20])
    .style(theme::button_style::primary)
    .on_press(Message::SaveSettings);

    let reset_btn = button(
        row![
            text("↩").size(14),
            Space::with_width(8),
            text("Reset to Defaults").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 20])
    .style(theme::button_style::secondary)
    .on_press(Message::ResetSettings);

    let saved_indicator = if app.settings_saved {
        container(
            row![
                text("✓").size(14).style(theme::text_style::success()),
                Space::with_width(8),
                text("Settings saved!")
                    .size(14)
                    .style(theme::text_style::success()),
            ]
            .align_y(Alignment::Center),
        )
    } else {
        container(Space::with_height(0))
    };

    let buttons = row![save_btn, Space::with_width(12), reset_btn, Space::with_width(24), saved_indicator,]
        .align_y(Alignment::Center);

    // About section
    let about_section = container(
        column![
            text("About zOpenTimestamps").size(16),
            Space::with_height(12),
            info_row("Version", "0.1.1"),
            info_row("License", "MIT"),
            info_row("Author", "AbdelStark"),
            Space::with_height(16),
            text("zOpenTimestamps is a Zcash blockchain timestamping tool inspired by OpenTimestamps. It creates cryptographic proofs that data existed at a specific point in time using Zcash's shielded transactions.")
                .size(12)
                .style(theme::text_style::muted()),
            Space::with_height(16),
            row![
                button(
                    row![
                        text(">").size(14),
                        Space::with_width(8),
                        text("GitHub").size(13),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([8, 16])
                .style(theme::button_style::secondary)
                .on_press(Message::OpenExplorer(
                    "https://github.com/AbdelStark/zopentimestamps".to_string()
                )),
            ],
        ]
        .padding(20),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    // Keyboard shortcuts
    let shortcuts_section = container(
        column![
            text("Environment Variables").size(16),
            Space::with_height(12),
            env_var_row("ZOTS_SEED", "BIP-39 seed phrase for wallet"),
            env_var_row("ZOTS_LIGHTWALLETD_URL", "Override lightwalletd server"),
            env_var_row("ZOTS_EXPLORER_URL", "Override block explorer URL"),
        ]
        .padding(20),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    column![
        title,
        Space::with_height(8),
        description,
        Space::with_height(24),
        explorer_section,
        Space::with_height(16),
        lightwalletd_section,
        Space::with_height(24),
        buttons,
        Space::with_height(24),
        shortcuts_section,
        Space::with_height(16),
        about_section,
    ]
    .width(Length::Fill)
    .into()
}

fn info_row<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    container(
        row![
            text(format!("{label}:"))
                .size(13)
                .style(theme::text_style::muted())
                .width(80),
            text(value).size(13),
        ]
        .align_y(Alignment::Center)
        .padding([4, 0]),
    )
    .into()
}

fn env_var_row<'a>(name: &'a str, description: &'a str) -> Element<'a, Message> {
    container(
        row![
            text(name)
                .size(12)
                .style(theme::text_style::accent())
                .width(200),
            text(description).size(12).style(theme::text_style::dim()),
        ]
        .align_y(Alignment::Center)
        .padding([6, 0]),
    )
    .into()
}
