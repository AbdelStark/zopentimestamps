//! History view - Transaction history

use crate::app::IkkiApp;
use crate::components::{card, empty_state, loading_spinner, transaction_item};
use crate::message::Message;
use crate::theme::{self, spacing, typography};
use iced::widget::{Space, column, row, text};
use iced::{Element, Length};

/// History view showing all past transactions
pub fn view(app: &IkkiApp) -> Element<Message> {
    let title = text("Activity")
        .size(typography::H2)
        .style(theme::text_style::primary());

    let subtitle = text("Your transaction history")
        .size(typography::BODY_SMALL)
        .style(theme::text_style::secondary());

    // Transaction list
    let transaction_list: Element<Message> = if app.transactions_loading {
        loading_spinner("Loading transactions...")
    } else if app.transactions.is_empty() {
        empty_state(
            "...",
            "No transactions yet",
            "Send or receive ZEC to see your activity here",
        )
    } else {
        let items: Vec<Element<Message>> = app
            .transactions
            .iter()
            .map(|tx| transaction_item(tx))
            .collect();

        card(column(items).spacing(spacing::XXS))
    };

    column![
        row![title,],
        Space::with_height(spacing::XXS),
        subtitle,
        Space::with_height(spacing::LG),
        transaction_list,
    ]
    .width(Length::Fill)
    .max_width(600)
    .into()
}
