//! Send view - Send ZEC to an address

use crate::app::IkkiApp;
use crate::components::{card, divider, info_row};
use crate::message::{Message, SendPhase, View};
use crate::theme::{self, colors, format_zec, radius, spacing, typography};
use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

/// Send view with multi-step flow
///
/// Steps:
/// 1. Input - Enter address, amount, and optional memo
/// 2. Preview - Review transaction details
/// 3. Sending - Show progress
/// 4. Complete - Show success with txid
pub fn view(app: &IkkiApp) -> Element<Message> {
    match &app.send_state.phase {
        SendPhase::Input | SendPhase::Failed => input_view(app),
        SendPhase::Preview => preview_view(app),
        SendPhase::Sending => sending_view(app),
        SendPhase::Complete { txid } => complete_view(app, txid),
    }
}

fn input_view(app: &IkkiApp) -> Element<Message> {
    let title = text("Send ZEC")
        .size(typography::H2)
        .style(theme::text_style::primary());

    let subtitle = text("Send shielded ZEC to any Zcash address")
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    // Balance display
    let balance_zec = format_zec(app.balance);
    let balance_row = row![
        text("Available:")
            .size(typography::BODY_SMALL)
            .style(theme::text_style::secondary()),
        Space::with_width(spacing::XS),
        text(format!("{balance_zec} ZEC"))
            .size(typography::BODY_SMALL)
            .style(theme::text_style::primary()),
    ]
    .align_y(Alignment::Center);

    // Amount input with max button
    let amount_label = text("Amount")
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    let amount_input = text_input("0.00", &app.send_state.amount)
        .padding(spacing::MD)
        .size(typography::H3)
        .style(theme::input_style::large)
        .width(Length::Fill)
        .on_input(Message::SendAmountChanged);

    let max_btn = button(
        text("MAX")
            .size(typography::CAPTION)
            .style(theme::text_style::brand()),
    )
    .padding([spacing::XS, spacing::SM])
    .style(theme::button_style::ghost)
    .on_press(Message::SendMaxAmount);

    let zec_suffix = text("ZEC")
        .size(typography::BODY)
        .style(theme::text_style::secondary());

    let amount_row = row![
        amount_input,
        Space::with_width(spacing::XS),
        zec_suffix,
        max_btn,
    ]
    .align_y(Alignment::Center);

    // Address input
    let address_label = text("To address")
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    let address_input = text_input("Enter Zcash address...", &app.send_state.address)
        .padding(spacing::MD)
        .size(typography::BODY)
        .style(theme::input_style::default)
        .width(Length::Fill)
        .on_input(Message::SendAddressChanged);

    // Memo input (optional)
    let memo_label = row![
        text("Memo")
            .size(typography::BODY_SMALL)
            .style(theme::text_style::secondary()),
        Space::with_width(spacing::XS),
        text("(optional)")
            .size(typography::CAPTION)
            .style(theme::text_style::tertiary()),
    ];

    let memo_input = text_input("Add a private message...", &app.send_state.memo)
        .padding(spacing::MD)
        .size(typography::BODY)
        .style(theme::input_style::default)
        .width(Length::Fill)
        .on_input(Message::SendMemoChanged);

    // Error display
    let error_element: Element<Message> = if let Some(error) = &app.send_state.error {
        container(
            row![
                text("!")
                    .size(typography::BODY)
                    .style(theme::text_style::error()),
                Space::with_width(spacing::SM),
                text(error)
                    .size(typography::BODY_SMALL)
                    .style(theme::text_style::error()),
            ]
            .padding(spacing::SM),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color {
                a: 0.1,
                ..colors::ERROR
            })),
            border: iced::Border {
                radius: radius::MD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Preview button
    let preview_btn = button(
        row![text("Review Transaction").size(typography::BODY),]
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .padding([spacing::MD, spacing::LG])
    .width(Length::Fill)
    .style(theme::button_style::primary_large)
    .on_press(Message::SendPreview);

    // Cancel button
    let cancel_btn = button(text("Cancel").size(typography::BODY).center())
        .padding([spacing::SM, spacing::LG])
        .style(theme::button_style::ghost)
        .on_press(Message::NavigateTo(View::Home));

    let form_content = column![
        amount_label,
        Space::with_height(spacing::XS),
        amount_row,
        Space::with_height(spacing::LG),
        address_label,
        Space::with_height(spacing::XS),
        address_input,
        Space::with_height(spacing::LG),
        memo_label,
        Space::with_height(spacing::XS),
        memo_input,
        Space::with_height(spacing::MD),
        error_element,
    ];

    column![
        title,
        Space::with_height(spacing::XXS),
        subtitle,
        Space::with_height(spacing::SM),
        balance_row,
        Space::with_height(spacing::LG),
        card(form_content),
        Space::with_height(spacing::LG),
        preview_btn,
        Space::with_height(spacing::SM),
        cancel_btn,
    ]
    .width(Length::Fill)
    .max_width(500)
    .into()
}

fn preview_view(app: &IkkiApp) -> Element<Message> {
    let title = text("Review Transaction")
        .size(typography::H2)
        .style(theme::text_style::primary());

    let amount: f64 = app.send_state.amount.parse().unwrap_or(0.0);
    let zatoshi = (amount * 100_000_000.0) as u64;
    let fee = app.send_state.fee_estimate;
    let total = zatoshi + fee;

    // Amount display
    let amount_display = column![
        text(format!("{amount:.8}"))
            .size(typography::DISPLAY)
            .style(theme::text_style::primary()),
        text("ZEC")
            .size(typography::H3)
            .style(theme::text_style::secondary()),
    ]
    .align_x(Alignment::Center);

    // Transaction details
    let details = column![
        info_row(
            "To",
            crate::theme::truncate_address(&app.send_state.address, 12)
        ),
        divider(),
        info_row("Amount", format!("{} ZEC", format_zec(zatoshi))),
        divider(),
        info_row("Network fee", format!("{} ZEC", format_zec(fee))),
        divider(),
        info_row("Total", format!("{} ZEC", format_zec(total))),
    ];

    let memo_section: Element<Message> = if !app.send_state.memo.is_empty() {
        column![
            Space::with_height(spacing::MD),
            divider(),
            info_row("Memo", app.send_state.memo.clone()),
        ]
        .into()
    } else {
        Space::with_height(0).into()
    };

    // Buttons
    let confirm_btn =
        button(row![text("Confirm & Send").size(typography::BODY),].width(Length::Fill))
            .padding([spacing::MD, spacing::LG])
            .width(Length::Fill)
            .style(theme::button_style::send)
            .on_press(Message::SendConfirm);

    let back_btn = button(text("Edit").size(typography::BODY).center())
        .padding([spacing::SM, spacing::LG])
        .style(theme::button_style::ghost)
        .on_press(Message::SendCancel);

    column![
        title,
        Space::with_height(spacing::XL),
        container(amount_display)
            .width(Length::Fill)
            .center_x(Length::Fill),
        Space::with_height(spacing::XL),
        card(column![details, memo_section]),
        Space::with_height(spacing::LG),
        confirm_btn,
        Space::with_height(spacing::SM),
        back_btn,
    ]
    .width(Length::Fill)
    .max_width(500)
    .into()
}

fn sending_view(app: &IkkiApp) -> Element<Message> {
    let spinner_frames = ["...", "..", ".", ".."];
    let spinner = spinner_frames[app.spinner_frame % spinner_frames.len()];

    let content = column![
        text(spinner)
            .size(typography::DISPLAY)
            .style(theme::text_style::brand()),
        Space::with_height(spacing::LG),
        text("Sending ZEC")
            .size(typography::H2)
            .style(theme::text_style::primary()),
        Space::with_height(spacing::XS),
        text("Please wait while your transaction is being broadcast...")
            .size(typography::BODY)
            .style(theme::text_style::secondary()),
    ]
    .align_x(Alignment::Center)
    .padding(spacing::XXL);

    container(content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

fn complete_view<'a>(_app: &'a IkkiApp, txid: &'a str) -> Element<'a, Message> {
    let content = column![
        text("*").size(64.0).style(theme::text_style::success()),
        Space::with_height(spacing::LG),
        text("Transaction Sent!")
            .size(typography::H2)
            .style(theme::text_style::primary()),
        Space::with_height(spacing::XS),
        text("Your transaction has been broadcast to the network")
            .size(typography::BODY)
            .style(theme::text_style::secondary()),
        Space::with_height(spacing::XL),
        card(column![
            text("Transaction ID")
                .size(typography::CAPTION)
                .style(theme::text_style::secondary()),
            Space::with_height(spacing::XXS),
            text(crate::theme::truncate_address(txid, 16))
                .size(typography::BODY_SMALL)
                .style(theme::text_style::primary()),
        ]),
        Space::with_height(spacing::XL),
        button(text("Done").size(typography::BODY).center())
            .padding([spacing::MD, spacing::XL])
            .width(200)
            .style(theme::button_style::primary)
            .on_press(Message::SendReset),
    ]
    .align_x(Alignment::Center)
    .padding(spacing::XL);

    container(content)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
}
