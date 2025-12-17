//! Verify view - Verify timestamp proofs

use crate::app::ZotsApp;
use crate::message::Message;
use crate::theme::{self, colors};
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = row![
        text(">").size(28),
        Space::with_width(12),
        text("Verify Proof").size(24),
    ]
    .align_y(Alignment::Center);

    let description = text("Verify a timestamp proof against the Zcash blockchain. Optionally provide the original file to check the hash matches.")
        .size(14)
        .style(theme::text_style::muted());

    // File/hash input (optional)
    let file_label = text("Original file or hash (optional)").size(14);

    let file_input = text_input(
        "Enter file path or hash to verify against...",
        &app.verify_file_input,
    )
    .padding(12)
    .size(14)
    .style(theme::input_style::default)
    .on_input(Message::VerifyFileInputChanged);

    let browse_file_btn = button(
        row![
            text("...").size(14),
            Space::with_width(8),
            text("Browse").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .style(theme::button_style::secondary)
    .on_press(Message::SelectVerifyFile);

    let file_row = row![file_input, Space::with_width(12), browse_file_btn,]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // Proof file input (required)
    let proof_label = text("Proof file (.zots)").size(14);

    let proof_input = text_input("Select a .zots proof file...", &app.verify_proof_input)
        .padding(12)
        .size(14)
        .style(theme::input_style::default)
        .on_input(Message::VerifyProofInputChanged);

    let browse_proof_btn = button(
        row![
            text("...").size(14),
            Space::with_width(8),
            text("Browse").size(14),
        ]
        .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .style(theme::button_style::secondary)
    .on_press(Message::SelectProofFile);

    let proof_row = row![proof_input, Space::with_width(12), browse_proof_btn,]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // Verify button
    let verify_btn = if app.verifying {
        button(
            row![
                text(app.spinner()).size(16),
                Space::with_width(12),
                text("Verifying...").size(14),
            ]
            .align_y(Alignment::Center),
        )
        .padding([14, 24])
        .style(theme::button_style::primary)
        .width(Length::Fill)
    } else {
        button(
            row![
                text("✓").size(16),
                Space::with_width(12),
                text("Verify Proof").size(14),
            ]
            .align_y(Alignment::Center),
        )
        .padding([14, 24])
        .style(theme::button_style::primary)
        .on_press(Message::StartVerify)
        .width(Length::Fill)
    };

    // Result section
    let result_section = if let Some(result) = &app.verify_result {
        let (icon, title_text, title_color) = if result.valid {
            ("✓", "Proof Valid!", colors::SUCCESS)
        } else {
            ("✗", "Verification Failed", colors::ERROR)
        };

        let hash_match_indicator = match result.file_hash_matches {
            Some(true) => Some(
                row![
                    text("✓").size(14).style(theme::text_style::success()),
                    Space::with_width(8),
                    text("File hash matches proof")
                        .size(13)
                        .style(theme::text_style::success()),
                ]
                .align_y(Alignment::Center),
            ),
            Some(false) => Some(
                row![
                    text("✗").size(14).style(theme::text_style::error()),
                    Space::with_width(8),
                    text("File hash does NOT match!")
                        .size(13)
                        .style(theme::text_style::error()),
                ]
                .align_y(Alignment::Center),
            ),
            None => None,
        };

        let error_color = if result.valid {
            colors::WARNING
        } else {
            colors::ERROR
        };
        let error_text = result
            .error
            .as_ref()
            .map(|e| text(e).size(13).color(error_color));

        let mut content_col = column![
            row![
                text(icon).size(20).color(title_color),
                Space::with_width(12),
                text(title_text).size(16).color(title_color),
            ]
            .align_y(Alignment::Center),
        ];

        if let Some(indicator) = hash_match_indicator {
            content_col = content_col.push(Space::with_height(12));
            content_col = content_col.push(indicator);
        }

        if let Some(err) = error_text {
            content_col = content_col.push(Space::with_height(12));
            content_col = content_col.push(err);
        }

        content_col = content_col.push(Space::with_height(16));
        content_col = content_col.push(info_row("Hash", result.hash.clone()));
        content_col = content_col.push(info_row("Algorithm", result.algorithm.name().to_string()));

        if !result.network.is_empty() {
            content_col = content_col.push(info_row("Network", result.network.clone()));
        }
        if result.block_height > 0 {
            content_col = content_col.push(info_row("Block", result.block_height.to_string()));
        }
        if !result.timestamp.is_empty() {
            content_col = content_col.push(info_row("Timestamp", result.timestamp.clone()));
        }
        if !result.txid.is_empty() {
            content_col = content_col.push(info_row("Transaction", result.txid.clone()));
        }

        content_col = content_col.push(Space::with_height(16));

        let copy_btn = button(
            row![
                text(">").size(14),
                Space::with_width(8),
                text("Copy Compact").size(13),
            ]
            .align_y(Alignment::Center),
        )
        .padding([10, 16])
        .style(theme::button_style::secondary)
        .on_press(Message::CopyToClipboard(result.compact.clone()));

        let mut buttons_row = row![copy_btn];

        if !result.explorer_link.is_empty() {
            buttons_row = buttons_row.push(Space::with_width(12));
            buttons_row = buttons_row.push(
                button(
                    row![
                        text(">").size(14),
                        Space::with_width(8),
                        text("View in Explorer").size(13),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([10, 16])
                .style(theme::button_style::secondary)
                .on_press(Message::OpenExplorer(result.explorer_link.clone())),
            );
        }

        content_col = content_col.push(buttons_row);

        let border_color = if result.valid {
            theme::colors::SUCCESS
        } else {
            theme::colors::ERROR
        };

        container(content_col.padding(20))
            .style(move |theme| {
                let mut style = theme::container_style::card(theme);
                style.border.color = border_color;
                style
            })
            .width(Length::Fill)
    } else if let Some(error) = &app.verify_error {
        container(
            column![
                row![
                    text("✗").size(20).style(theme::text_style::error()),
                    Space::with_width(12),
                    text("Error").size(16).style(theme::text_style::error()),
                ]
                .align_y(Alignment::Center),
                Space::with_height(12),
                text(error).size(13).style(theme::text_style::muted()),
            ]
            .padding(20),
        )
        .style(|theme| {
            let mut style = theme::container_style::card(theme);
            style.border.color = theme::colors::ERROR;
            style
        })
        .width(Length::Fill)
    } else {
        container(Space::with_height(0))
    };

    // Main content
    let content = container(
        column![
            file_label,
            Space::with_height(8),
            file_row,
            Space::with_height(24),
            proof_label,
            Space::with_height(8),
            proof_row,
            Space::with_height(24),
            verify_btn,
        ]
        .padding(24),
    )
    .style(theme::container_style::card)
    .width(Length::Fill);

    column![
        title,
        Space::with_height(8),
        description,
        Space::with_height(24),
        content,
        Space::with_height(24),
        result_section,
    ]
    .width(Length::Fill)
    .into()
}

fn info_row(label: &'static str, value: String) -> Element<'static, Message> {
    let display_value = if value.len() > 50 {
        format!("{}...{}", &value[..24], &value[value.len() - 12..])
    } else {
        value
    };

    container(
        row![
            text(format!("{label}:"))
                .size(13)
                .style(theme::text_style::muted())
                .width(100),
            text(display_value)
                .size(13)
                .style(theme::text_style::accent()),
        ]
        .align_y(Alignment::Center)
        .padding([4, 0]),
    )
    .into()
}
