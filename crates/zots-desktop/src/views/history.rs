//! History view - List past proofs

use crate::app::ZotsApp;
use crate::message::Message;
use crate::theme::{self, colors};
use iced::widget::{Space, button, column, container, horizontal_space, row, scrollable, text};
use iced::{Alignment, Color, Element, Length};

pub fn view(app: &ZotsApp) -> Element<Message> {
    let title = row![
        text(">").size(28),
        Space::with_width(12),
        text("Proof History").size(24),
    ]
    .align_y(Alignment::Center);

    let description =
        text("View and manage your timestamp proofs (.zots files in current directory)")
            .size(14)
            .style(theme::text_style::muted());

    let refresh_btn = button(
        row![
            text(if app.history_loading {
                app.spinner()
            } else {
                ">"
            })
            .size(14),
            Space::with_width(8),
            text(if app.history_loading {
                "Loading..."
            } else {
                "Refresh"
            })
            .size(14),
        ]
        .align_y(Alignment::Center),
    )
    .padding([10, 16])
    .style(theme::button_style::secondary)
    .on_press(Message::LoadHistory);

    let header = row![title, horizontal_space(), refresh_btn,].align_y(Alignment::Center);

    // History list
    let content = if app.history.is_empty() && !app.history_loading {
        container(
            column![
                text("--").size(48),
                Space::with_height(16),
                text("No proofs found")
                    .size(16)
                    .style(theme::text_style::muted()),
                Space::with_height(8),
                text("Create a timestamp to see it here")
                    .size(13)
                    .style(theme::text_style::dim()),
            ]
            .align_x(Alignment::Center)
            .padding(48),
        )
        .style(theme::container_style::card)
        .width(Length::Fill)
    } else if app.history_loading {
        container(
            column![
                text(app.spinner()).size(32),
                Space::with_height(16),
                text("Loading proofs...")
                    .size(14)
                    .style(theme::text_style::muted()),
            ]
            .align_x(Alignment::Center)
            .padding(48),
        )
        .style(theme::container_style::card)
        .width(Length::Fill)
    } else {
        let entries: Vec<Element<Message>> = app
            .history
            .iter()
            .map(|entry| {
                let status_icon = if entry.confirmed { "✓" } else { "⏳" };
                let status_color = if entry.confirmed {
                    colors::SUCCESS
                } else {
                    colors::WARNING
                };

                let filename = entry
                    .path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                let hash_display = if entry.hash.len() > 20 {
                    format!(
                        "{}...{}",
                        &entry.hash[..10],
                        &entry.hash[entry.hash.len() - 8..]
                    )
                } else {
                    entry.hash.clone()
                };

                let details = if entry.confirmed {
                    format!(
                        "{} · Block {}",
                        entry.network.as_deref().unwrap_or("Unknown"),
                        entry.block_height.unwrap_or(0)
                    )
                } else {
                    "Pending confirmation".to_string()
                };

                let created = entry.created.clone();
                container(
                    row![
                        // Status indicator
                        text(status_icon).size(18).color(status_color),
                        Space::with_width(16),
                        // Info column
                        column![
                            text(filename).size(14),
                            Space::with_height(4),
                            text(hash_display)
                                .size(12)
                                .style(theme::text_style::accent()),
                            Space::with_height(2),
                            text(details).size(11).style(theme::text_style::dim()),
                        ]
                        .width(Length::Fill),
                        // Date
                        text(created).size(11).style(theme::text_style::muted()),
                        Space::with_width(16),
                        // Actions
                        button(text("x").size(12))
                            .padding([6, 10])
                            .style(theme::button_style::secondary)
                            .on_press(Message::DeleteProof(entry.path.clone())),
                    ]
                    .align_y(Alignment::Center)
                    .padding(16),
                )
                .style(theme::container_style::surface)
                .width(Length::Fill)
                .into()
            })
            .collect();

        container(scrollable(column(entries).spacing(8).padding(4)).height(Length::Fill))
            .style(theme::container_style::card)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
    };

    // Summary stats
    let confirmed_count = app.history.iter().filter(|e| e.confirmed).count();
    let pending_count = app.history.len() - confirmed_count;

    let stats = row![
        stat_badge("Total", app.history.len(), colors::TEXT_MUTED),
        Space::with_width(16),
        stat_badge("Confirmed", confirmed_count, colors::SUCCESS),
        Space::with_width(16),
        stat_badge("Pending", pending_count, colors::WARNING),
    ];

    column![
        header,
        Space::with_height(8),
        description,
        Space::with_height(16),
        stats,
        Space::with_height(16),
        content,
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn stat_badge<'a>(label: &'a str, count: usize, color: Color) -> Element<'a, Message> {
    container(
        row![
            text(format!("{label}: "))
                .size(12)
                .style(theme::text_style::dim()),
            text(count.to_string()).size(12).color(color),
        ]
        .align_y(Alignment::Center),
    )
    .into()
}
