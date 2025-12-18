//! Ikki library - Tauri application setup and commands

mod commands;
mod state;

use state::AppState;

pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Wallet commands
            commands::wallet::check_wallet_exists,
            commands::wallet::init_wallet,
            commands::wallet::load_wallet,
            commands::wallet::auto_load_wallet,
            commands::wallet::reset_wallet,
            commands::wallet::get_balance,
            commands::wallet::get_address,
            commands::wallet::get_new_address,
            commands::wallet::get_all_addresses,
            commands::wallet::sync_wallet,
            commands::wallet::generate_seed,
            // Transaction commands
            commands::transactions::send_transaction,
            commands::transactions::get_transactions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
