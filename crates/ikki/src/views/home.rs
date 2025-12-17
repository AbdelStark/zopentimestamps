//! Home view - Main dashboard with account card and actions

use crate::app::IkkiApp;
use crate::components::{
    ActionType, account_card, action_button, section_header, transaction_item,
};
use crate::message::{Message, View};
use crate::theme::{self, spacing};
use iced::widget::{Space, column, container, row};
use iced::{Alignment, Element, Length};

/// Home view displaying the main wallet dashboard
///
/// Layout inspired by Revolut:
/// - Account card at top showing balance
/// - Action buttons (Send, Receive) below the card
/// - Recent activity section with latest transactions
pub fn view(app: &IkkiApp) -> Element<Message> {
    let network = app
        .config
        .as_ref()
        .map(|c| c.network.name())
        .unwrap_or("Testnet");

    // Account card
    let card = account_card(app.balance, &app.address, network, app.sync_state.syncing);

    // Action buttons row
    let actions = container(
        row![
            action_button(ActionType::Send, Message::NavigateTo(View::Send)),
            Space::with_width(spacing::XXL),
            action_button(ActionType::Receive, Message::NavigateTo(View::Receive)),
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .center_x(Length::Fill);

    // Recent transactions section
    let recent_header = section_header(
        "Recent Activity",
        if app.transactions.is_empty() {
            None
        } else {
            Some(("View all", Message::NavigateTo(View::History)))
        },
    );

    let recent_transactions: Element<Message> = if app.transactions.is_empty() {
        crate::components::empty_state(
            "...",
            "No activity yet",
            "Your transactions will appear here",
        )
    } else {
        let items: Vec<Element<Message>> = app
            .transactions
            .iter()
            .take(5)
            .map(|tx| transaction_item(tx))
            .collect();

        container(column(items).spacing(spacing::XXS))
            .style(theme::container_style::card)
            .width(Length::Fill)
            .into()
    };

    // Main layout
    column![
        card,
        Space::with_height(spacing::XL),
        actions,
        Space::with_height(spacing::XXL),
        recent_header,
        Space::with_height(spacing::MD),
        recent_transactions,
    ]
    .spacing(0)
    .width(Length::Fill)
    .max_width(600)
    .into()
}
