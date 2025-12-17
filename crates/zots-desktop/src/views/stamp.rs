//! Stamp view - Create timestamps

use crate::app::ZotsApp;
use crate::message::Message;
use crate::theme;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = row![
        text("📝").size(28),
        Space::with_width(12),
        text("Create Timestamp").size(24),
    ]
    .align_y(Alignment::Center);

    let description = text("Timestamp a file or hash on the Zcash blockchain. The proof will be saved as a .zots file.")
        .size(14)
        .style(theme::text_style::muted());

    // Input section
    let input_label = text("File path or hash").size(14);

    let file_input = text_input("Enter file path or 64-char hex hash...", &app.stamp_input)
        .padding(12)
        .size(14)
        .style(theme::input_style::default)
        .on_input(Message::StampInputChanged);

    let browse_btn = button(
        row![text("📁").size(14), Space::with_width(8), text("Browse").size(14),]
            .align_y(Alignment::Center),
    )
    .padding([12, 16])
    .style(theme::button_style::secondary)
    .on_press(Message::SelectFile);

    let input_row = row![file_input, Space::with_width(12), browse_btn,]
        .align_y(Alignment::Center)
        .width(Length::Fill);

    // Algorithm selection
    let algo_label = text("Hash Algorithm").size(14);
    let algo_value = text(app.hash_algorithm.name())
        .size(14)
        .style(theme::text_style::accent());

    let toggle_btn = button(text("Toggle").size(12))
        .padding([8, 12])
        .style(theme::button_style::secondary)
        .on_press(Message::ToggleAlgorithm);

    let algo_row = row![
        algo_label,
        Space::with_width(16),
        algo_value,
        Space::with_width(16),
        toggle_btn,
    ]
    .align_y(Alignment::Center);

    // Stamp button
    let stamp_btn = if app.stamp_phase.is_busy() {
        button(
            row![
                text(app.spinner()).size(16),
                Space::with_width(12),
                text(app.stamp_phase.message()).size(14),
            ]
            .align_y(Alignment::Center),
        )
        .padding([14, 24])
        .style(theme::button_style::primary)
        .width(Length::Fill)
    } else {
        button(
            row![
                text("⏰").size(16),
                Space::with_width(12),
                text("Create Timestamp").size(14),
            ]
            .align_y(Alignment::Center),
        )
        .padding([14, 24])
        .style(theme::button_style::primary)
        .on_press(Message::StartStamp)
        .width(Length::Fill)
    };

    // Result section
    let result_section = if let Some(result) = &app.stamp_result {
        let block_str = result.block_height.to_string();
        let output_str = result.output_path.display().to_string();
        container(
            column![
                row![
                    text("✓").size(20).style(theme::text_style::success()),
                    Space::with_width(12),
                    text("Timestamp Created Successfully!")
                        .size(16)
                        .style(theme::text_style::success()),
                ]
                .align_y(Alignment::Center),
                Space::with_height(16),
                info_row("Hash", result.hash.clone(), true),
                info_row("Algorithm", result.algorithm.name().to_string(), false),
                info_row("Transaction", result.txid.clone(), true),
                info_row("Block", block_str, false),
                info_row("Saved to", output_str, false),
                Space::with_height(16),
                row![
                    button(
                        row![text("📋").size(14), Space::with_width(8), text("Copy Proof").size(13),]
                            .align_y(Alignment::Center),
                    )
                    .padding([10, 16])
                    .style(theme::button_style::secondary)
                    .on_press(Message::CopyToClipboard(result.compact.clone())),
                    Space::with_width(12),
                    button(
                        row![text("🔗").size(14), Space::with_width(8), text("View in Explorer").size(13),]
                            .align_y(Alignment::Center),
                    )
                    .padding([10, 16])
                    .style(theme::button_style::secondary)
                    .on_press(Message::OpenExplorer(result.explorer_link.clone())),
                ],
            ]
            .padding(20),
        )
        .style(theme::container_style::card)
        .width(Length::Fill)
    } else if let Some(error) = &app.stamp_error {
        container(
            column![
                row![
                    text("✗").size(20).style(theme::text_style::error()),
                    Space::with_width(12),
                    text("Stamp Failed").size(16).style(theme::text_style::error()),
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
            input_label,
            Space::with_height(8),
            input_row,
            Space::with_height(24),
            algo_row,
            Space::with_height(24),
            stamp_btn,
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

fn info_row(label: &'static str, value: String, copyable: bool) -> Element<'static, Message> {
    let display_value = if value.len() > 40 {
        format!("{}...{}", &value[..20], &value[value.len() - 12..])
    } else {
        value.clone()
    };
    let value_text = text(display_value).size(13).style(theme::text_style::accent());

    let mut r = row![
        text(format!("{label}:"))
            .size(13)
            .style(theme::text_style::muted())
            .width(100),
        value_text,
    ]
    .align_y(Alignment::Center);

    if copyable {
        r = r.push(Space::with_width(12));
        r = r.push(
            button(text("📋").size(11))
                .padding([4, 8])
                .style(theme::button_style::secondary)
                .on_press(Message::CopyToClipboard(value)),
        );
    }

    container(r.padding([4, 0])).into()
}
