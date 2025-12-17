//! Transaction item component - Displays a transaction in a list

use crate::message::{Message, Transaction, TransactionStatus, TransactionType};
use crate::theme::{self, colors, format_zec, radius, spacing, truncate_address, typography};
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

/// Creates a transaction list item
///
/// Clean, readable design showing:
/// - Transaction type icon
/// - Direction/type label
/// - Address or memo preview
/// - Amount with color coding
/// - Status indicator
pub fn transaction_item<'a>(tx: &'a Transaction) -> Element<'a, Message> {
    // Icon based on transaction type
    let icon = match tx.tx_type {
        TransactionType::Sent => "^",
        TransactionType::Received => "v",
        TransactionType::Shielding => "*",
        TransactionType::Internal => "o",
    };

    let icon_bg = match tx.tx_type {
        TransactionType::Sent => colors::SEND,
        TransactionType::Received => colors::RECEIVE,
        TransactionType::Shielding => colors::SHIELDED,
        TransactionType::Internal => colors::TEXT_TERTIARY,
    };

    let icon_element = container(
        text(icon)
            .size(typography::BODY)
            .style(theme::text_style::on_dark()),
    )
    .width(40)
    .height(40)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(icon_bg)),
        border: iced::Border {
            radius: radius::FULL.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Title and subtitle
    let title = text(tx.tx_type.label())
        .size(typography::BODY)
        .style(theme::text_style::primary());

    let subtitle_text = if let Some(addr) = &tx.address {
        truncate_address(addr, 8)
    } else if let Some(memo) = &tx.memo {
        if memo.len() > 30 {
            format!("{}...", &memo[..30])
        } else {
            memo.clone()
        }
    } else {
        format_timestamp(tx.timestamp)
    };

    let subtitle = text(subtitle_text)
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    // Amount
    let is_negative = tx.amount < 0;
    let amount_value = tx.amount.unsigned_abs();
    let amount_text = if is_negative {
        format!("-{} ZEC", format_zec(amount_value))
    } else {
        format!("+{} ZEC", format_zec(amount_value))
    };

    let amount_color = if is_negative {
        colors::SEND
    } else {
        colors::RECEIVE
    };

    let amount = text(amount_text)
        .size(typography::BODY)
        .style(move |_| text::Style {
            color: Some(amount_color),
        });

    // Status indicator
    let status_element: Element<'a, Message> = match tx.status {
        TransactionStatus::Pending => container(
            text("Pending")
                .size(typography::CAPTION)
                .style(theme::text_style::warning()),
        )
        .padding([spacing::XXXS, spacing::XS])
        .style(theme::container_style::warning_pill)
        .into(),
        TransactionStatus::Confirmed => Space::with_height(0).into(),
        TransactionStatus::Failed => container(
            text("Failed")
                .size(typography::CAPTION)
                .style(theme::text_style::error()),
        )
        .padding([spacing::XXXS, spacing::XS])
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: 0.1,
                ..colors::ERROR
            })),
            border: iced::Border {
                radius: radius::FULL.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into(),
    };

    let txid = tx.txid.clone();
    button(
        row![
            icon_element,
            Space::with_width(spacing::MD),
            column![title, Space::with_height(spacing::XXXS), subtitle,].width(Length::Fill),
            column![amount, Space::with_height(spacing::XXXS), status_element,]
                .align_x(Alignment::End),
        ]
        .align_y(Alignment::Center)
        .padding([spacing::SM, spacing::MD]),
    )
    .style(|_theme, status| {
        let bg = match status {
            button::Status::Active => iced::Color::TRANSPARENT,
            button::Status::Hovered => colors::SURFACE_SECONDARY,
            button::Status::Pressed => colors::SURFACE_SECONDARY,
            button::Status::Disabled => iced::Color::TRANSPARENT,
        };

        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: colors::TEXT_PRIMARY,
            border: iced::Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .width(Length::Fill)
    .on_press(Message::TransactionSelected(txid))
    .into()
}

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Local, Utc};

    let datetime = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).unwrap_or_else(Utc::now);
    let local: DateTime<Local> = datetime.into();
    let now = Local::now();

    let duration = now.signed_duration_since(local);

    if duration.num_minutes() < 1 {
        "Just now".to_string()
    } else if duration.num_hours() < 1 {
        format!("{} min ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else {
        local.format("%b %d, %Y").to_string()
    }
}

/// Transaction list with date grouping
#[allow(dead_code)]
pub fn transaction_list<'a>(transactions: &'a [Transaction]) -> Element<'a, Message> {
    if transactions.is_empty() {
        return crate::components::empty_state(
            "...",
            "No transactions yet",
            "Send or receive ZEC to see your activity here",
        );
    }

    let items: Vec<Element<'a, Message>> =
        transactions.iter().map(|tx| transaction_item(tx)).collect();

    column(items)
        .spacing(spacing::XXS)
        .width(Length::Fill)
        .into()
}
