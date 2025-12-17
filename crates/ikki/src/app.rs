//! Main application state and update logic for Ikki

use crate::message::{
    Message, NetworkChoice, OnboardingState, OnboardingStep, SendPhase, SendState, SyncState,
    Transaction, TransactionStatus, TransactionType, View,
};
use crate::theme::{self, colors, spacing, typography};
use crate::views;
use anyhow::Result;
use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Length, Subscription, Task};
use std::time::Duration;
use zots_zcash::ZcashConfig;

/// Main Ikki application state
pub struct IkkiApp {
    // Core state
    pub current_view: View,
    pub config: Option<ZcashConfig>,

    // Wallet state
    pub balance: u64,
    pub address: String,
    pub block_height: u64,
    pub sync_state: SyncState,

    // Send state
    pub send_state: SendState,

    // Transaction history
    pub transactions: Vec<Transaction>,
    pub transactions_loading: bool,

    // Settings
    pub lightwalletd_url: String,
    pub network: NetworkChoice,
    pub show_seed: bool,

    // Onboarding
    pub onboarding: OnboardingState,
    pub needs_onboarding: bool,

    // UI state
    pub spinner_frame: usize,
    pub toast: Option<(String, crate::message::ToastType)>,
    pub menu_open: bool,
}

impl Default for IkkiApp {
    fn default() -> Self {
        Self {
            current_view: View::Home,
            config: None,
            balance: 0,
            address: String::new(),
            block_height: 0,
            sync_state: SyncState::default(),
            send_state: SendState::default(),
            transactions: Vec::new(),
            transactions_loading: false,
            lightwalletd_url: "https://testnet.zec.rocks:443".to_string(),
            network: NetworkChoice::Testnet,
            show_seed: false,
            onboarding: OnboardingState::default(),
            needs_onboarding: true,
            spinner_frame: 0,
            toast: None,
            menu_open: false,
        }
    }
}

impl IkkiApp {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        // Load settings
        app.load_settings();

        // Try to load config from environment
        if let Ok(config) = ZcashConfig::from_env() {
            app.config = Some(config.clone());
            app.needs_onboarding = false;
            app.sync_state.syncing = true;

            // Start initial sync
            let task = Task::perform(initial_load(config), |result| match result {
                Ok((balance, address, block_height)) => Message::WalletLoaded {
                    balance,
                    address,
                    block_height,
                },
                Err(e) => Message::WalletLoadFailed(e.to_string()),
            });

            return (app, task);
        }

        // No wallet configured - show onboarding
        app.current_view = View::Onboarding;
        (app, Task::none())
    }

    fn title(&self) -> String {
        match self.current_view {
            View::Onboarding => "Ikki - Welcome".to_string(),
            _ => format!("Ikki - {}", self.current_view.title()),
        }
    }

    fn theme(&self) -> iced::Theme {
        theme::ikki_theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.sync_state.syncing || self.send_state.phase.is_busy() {
            iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation
            Message::NavigateTo(view) => {
                self.current_view = view;
                self.menu_open = false;

                if view == View::History && self.transactions.is_empty() {
                    return Task::perform(async {}, |_| Message::LoadTransactions);
                }
                Task::none()
            }

            // Wallet lifecycle
            Message::WalletLoaded {
                balance,
                address,
                block_height,
            } => {
                self.balance = balance;
                self.address = address;
                self.block_height = block_height;
                self.sync_state.syncing = false;
                self.sync_state.current_block = block_height;
                self.sync_state.target_block = block_height;
                Task::none()
            }
            Message::WalletLoadFailed(error) => {
                self.sync_state.syncing = false;
                self.sync_state.error = Some(error);
                Task::none()
            }

            // Sync
            Message::SyncWallet => {
                if let Some(config) = &self.config {
                    self.sync_state.syncing = true;
                    self.sync_state.error = None;
                    return Task::perform(sync_wallet(config.clone()), |result| match result {
                        Ok((height, balance)) => Message::SyncComplete {
                            block_height: height,
                            balance,
                        },
                        Err(e) => Message::SyncFailed(e.to_string()),
                    });
                }
                Task::none()
            }
            Message::SyncProgress { current, target } => {
                self.sync_state.current_block = current;
                self.sync_state.target_block = target;
                Task::none()
            }
            Message::SyncComplete {
                block_height,
                balance,
            } => {
                self.block_height = block_height;
                self.balance = balance;
                self.sync_state.syncing = false;
                self.sync_state.current_block = block_height;
                self.sync_state.target_block = block_height;
                Task::none()
            }
            Message::SyncFailed(error) => {
                self.sync_state.syncing = false;
                self.sync_state.error = Some(error);
                Task::none()
            }

            // Send
            Message::SendAmountChanged(amount) => {
                // Only allow valid numeric input
                if amount.is_empty()
                    || amount.parse::<f64>().is_ok()
                    || (amount.ends_with('.') && amount.matches('.').count() == 1)
                {
                    self.send_state.amount = amount;
                }
                Task::none()
            }
            Message::SendAddressChanged(address) => {
                self.send_state.address = address;
                Task::none()
            }
            Message::SendMemoChanged(memo) => {
                self.send_state.memo = memo;
                Task::none()
            }
            Message::SendMaxAmount => {
                // Calculate max sendable (balance minus estimated fee)
                let estimated_fee = 10_000u64; // 0.0001 ZEC
                let max = self.balance.saturating_sub(estimated_fee);
                let zec = max as f64 / 100_000_000.0;
                self.send_state.amount = format!("{zec:.8}");
                Task::none()
            }
            Message::SendPreview => {
                // Validate inputs
                if self.send_state.address.is_empty() {
                    self.send_state.error = Some("Please enter an address".to_string());
                    return Task::none();
                }
                if self.send_state.amount.is_empty() {
                    self.send_state.error = Some("Please enter an amount".to_string());
                    return Task::none();
                }

                let amount: f64 = match self.send_state.amount.parse() {
                    Ok(a) => a,
                    Err(_) => {
                        self.send_state.error = Some("Invalid amount".to_string());
                        return Task::none();
                    }
                };

                let zatoshi = (amount * 100_000_000.0) as u64;
                if zatoshi > self.balance {
                    self.send_state.error = Some("Insufficient balance".to_string());
                    return Task::none();
                }

                self.send_state.error = None;
                self.send_state.fee_estimate = 10_000; // 0.0001 ZEC
                self.send_state.phase = SendPhase::Preview;
                Task::none()
            }
            Message::SendConfirm => {
                if let Some(config) = &self.config {
                    self.send_state.phase = SendPhase::Sending;
                    self.send_state.error = None;

                    let amount: f64 = self.send_state.amount.parse().unwrap_or(0.0);
                    let zatoshi = (amount * 100_000_000.0) as u64;

                    return Task::perform(
                        send_transaction(
                            config.clone(),
                            self.send_state.address.clone(),
                            zatoshi,
                            self.send_state.memo.clone(),
                        ),
                        move |result| match result {
                            Ok((txid, fee)) => Message::SendComplete {
                                txid,
                                amount: zatoshi,
                                fee,
                            },
                            Err(e) => Message::SendFailed(e.to_string()),
                        },
                    );
                }
                Task::none()
            }
            Message::SendComplete { txid, amount, fee } => {
                self.send_state.phase = SendPhase::Complete { txid: txid.clone() };
                self.balance = self.balance.saturating_sub(amount + fee);

                // Add to transaction history
                self.transactions.insert(
                    0,
                    Transaction {
                        txid,
                        amount: -(amount as i64),
                        fee: Some(fee),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        block_height: None,
                        memo: if self.send_state.memo.is_empty() {
                            None
                        } else {
                            Some(self.send_state.memo.clone())
                        },
                        address: Some(self.send_state.address.clone()),
                        tx_type: TransactionType::Sent,
                        status: TransactionStatus::Pending,
                    },
                );

                Task::none()
            }
            Message::SendFailed(error) => {
                self.send_state.phase = SendPhase::Failed;
                self.send_state.error = Some(error);
                Task::none()
            }
            Message::SendCancel => {
                self.send_state.phase = SendPhase::Input;
                Task::none()
            }
            Message::SendReset => {
                self.send_state = SendState::default();
                self.current_view = View::Home;
                Task::none()
            }

            // Receive
            Message::GenerateNewAddress => {
                // For now, we use the same address
                // In a full implementation, this would generate a new diversified address
                Task::none()
            }
            Message::AddressGenerated(address) => {
                self.address = address;
                Task::none()
            }
            Message::CopyAddress => {
                Task::perform(async {}, |_| Message::CopyToClipboard(String::new()))
            }

            // Transactions
            Message::LoadTransactions => {
                self.transactions_loading = true;
                if let Some(config) = &self.config {
                    return Task::perform(load_transactions(config.clone()), |result| {
                        Message::TransactionsLoaded(result.unwrap_or_default())
                    });
                }
                Task::none()
            }
            Message::TransactionsLoaded(txs) => {
                self.transactions = txs;
                self.transactions_loading = false;
                Task::none()
            }
            Message::TransactionSelected(_txid) => {
                // TODO: Show transaction details modal
                Task::none()
            }

            // Settings
            Message::SeedInputChanged(seed) => {
                self.onboarding.seed_input = seed;
                Task::none()
            }
            Message::SaveSeed => {
                let seed = self.onboarding.seed_input.clone();
                match ZcashConfig::from_seed(&seed) {
                    Ok(config) => {
                        self.config = Some(config.clone());
                        self.needs_onboarding = false;
                        self.onboarding.seed_input.clear();
                        self.onboarding.step = OnboardingStep::Complete;
                        self.save_settings();
                        return Task::perform(initial_load(config), |result| match result {
                            Ok((balance, address, block_height)) => Message::WalletLoaded {
                                balance,
                                address,
                                block_height,
                            },
                            Err(e) => Message::WalletLoadFailed(e.to_string()),
                        });
                    }
                    Err(e) => {
                        self.onboarding.error = Some(e.to_string());
                    }
                }
                Task::none()
            }
            Message::SeedSaved => Task::none(),
            Message::SeedSaveFailed(error) => {
                self.onboarding.error = Some(error);
                Task::none()
            }
            Message::ShowSeed => {
                self.show_seed = true;
                Task::none()
            }
            Message::HideSeed => {
                self.show_seed = false;
                Task::none()
            }
            Message::LightwalletdChanged(url) => {
                self.lightwalletd_url = url;
                Task::none()
            }
            Message::NetworkChanged(network) => {
                self.network = network;
                Task::none()
            }
            Message::SaveSettings => {
                self.save_settings();
                Task::none()
            }

            // Onboarding
            Message::StartOnboarding => {
                self.onboarding.step = OnboardingStep::ImportOrCreate;
                Task::none()
            }
            Message::OnboardingImportSeed => {
                self.onboarding.step = OnboardingStep::ImportSeed;
                Task::none()
            }
            Message::OnboardingCreateNew => {
                self.onboarding.step = OnboardingStep::CreateSeed;
                Task::perform(generate_seed(), |result| match result {
                    Ok(seed) => Message::SeedGenerated(seed),
                    Err(_) => Message::SeedSaveFailed("Failed to generate seed".to_string()),
                })
            }
            Message::GenerateNewSeed => Task::perform(generate_seed(), |result| match result {
                Ok(seed) => Message::SeedGenerated(seed),
                Err(_) => Message::SeedSaveFailed("Failed to generate seed".to_string()),
            }),
            Message::SeedGenerated(seed) => {
                self.onboarding.generated_seed = Some(seed);
                Task::none()
            }
            Message::ConfirmSeedBackup => {
                self.onboarding.seed_confirmed = true;
                if let Some(seed) = &self.onboarding.generated_seed {
                    self.onboarding.seed_input = seed.clone();
                    return Task::perform(async {}, |_| Message::SaveSeed);
                }
                Task::none()
            }
            Message::OnboardingComplete => {
                self.needs_onboarding = false;
                self.current_view = View::Home;
                Task::none()
            }

            // UI
            Message::Tick => {
                self.spinner_frame = (self.spinner_frame + 1) % 8;
                Task::none()
            }
            Message::CopyToClipboard(text) => {
                let text_to_copy = if text.is_empty() {
                    self.address.clone()
                } else {
                    text
                };
                // Note: In a real implementation, we'd use the clipboard API
                self.toast = Some((
                    "Copied to clipboard".to_string(),
                    crate::message::ToastType::Success,
                ));
                let _ = text_to_copy; // Suppress unused warning
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| Message::DismissToast,
                )
            }
            Message::Copied => {
                self.toast = Some(("Copied!".to_string(), crate::message::ToastType::Success));
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| Message::DismissToast,
                )
            }
            Message::ShowToast(msg, toast_type) => {
                self.toast = Some((msg, toast_type));
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    },
                    |_| Message::DismissToast,
                )
            }
            Message::DismissToast => {
                self.toast = None;
                Task::none()
            }
            Message::OpenExplorer(url) => {
                let _ = open::that(&url);
                Task::none()
            }
            Message::ToggleMenu => {
                self.menu_open = !self.menu_open;
                Task::none()
            }
            Message::CloseMenu => {
                self.menu_open = false;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Onboarding view is full-screen
        if self.needs_onboarding || self.current_view == View::Onboarding {
            return self.onboarding_view();
        }

        let sidebar = self.sidebar();
        let content = match self.current_view {
            View::Home => views::home::view(self),
            View::Send => views::send::view(self),
            View::Receive => views::receive::view(self),
            View::History => views::history::view(self),
            View::Settings => views::settings::view(self),
            View::Onboarding => self.onboarding_view(),
        };

        let main_content = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([spacing::LG, spacing::XL])
            .style(theme::container_style::background);

        // Toast overlay
        let toast_overlay = if let Some((msg, toast_type)) = &self.toast {
            let bg_color = match toast_type {
                crate::message::ToastType::Success => colors::SUCCESS,
                crate::message::ToastType::Error => colors::ERROR,
                crate::message::ToastType::Warning => colors::WARNING,
                crate::message::ToastType::Info => colors::INFO,
            };

            container(
                container(
                    text(msg)
                        .size(typography::BODY_SMALL)
                        .style(theme::text_style::on_dark()),
                )
                .padding([spacing::SM, spacing::MD])
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg_color)),
                    border: iced::Border {
                        radius: theme::radius::MD.into(),
                        ..Default::default()
                    },
                    shadow: theme::shadows::md(),
                    ..Default::default()
                }),
            )
            .width(Length::Fill)
            .padding(spacing::LG)
            .center_x(Length::Fill)
        } else {
            container(Space::with_height(0))
        };

        let layout = row![sidebar, main_content]
            .width(Length::Fill)
            .height(Length::Fill);

        // Stack with toast
        column![layout, toast_overlay]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn sidebar(&self) -> Element<Message> {
        let nav_items = [
            (View::Home, "Home"),
            (View::Send, "Send"),
            (View::Receive, "Receive"),
            (View::History, "Activity"),
            (View::Settings, "Settings"),
        ];

        let nav_buttons: Vec<Element<Message>> = nav_items
            .iter()
            .map(|(view, label)| {
                let is_active = self.current_view == *view;
                button(
                    text(*label)
                        .size(typography::BODY_SMALL)
                        .width(Length::Fill),
                )
                .padding([spacing::SM, spacing::MD])
                .width(Length::Fill)
                .style(move |theme, status| theme::button_style::nav(theme, status, is_active))
                .on_press(Message::NavigateTo(*view))
                .into()
            })
            .collect();

        let logo = column![
            text("Ikki")
                .size(typography::H2)
                .style(theme::text_style::brand()),
            text("Zcash Wallet")
                .size(typography::CAPTION)
                .style(theme::text_style::secondary()),
        ]
        .padding([spacing::LG, spacing::MD]);

        let nav = column(nav_buttons)
            .spacing(spacing::XXS)
            .padding([spacing::XS, spacing::SM]);

        // Sync status at bottom
        let sync_status = crate::components::sync_indicator(
            self.sync_state.syncing,
            self.sync_state.progress(),
            self.block_height,
        );

        let bottom_section = column![
            crate::components::divider(),
            Space::with_height(spacing::SM),
            container(sync_status).padding([0.0, spacing::MD]),
            Space::with_height(spacing::SM),
        ];

        container(
            column![
                logo,
                Space::with_height(spacing::SM),
                nav,
                Space::with_height(Length::Fill),
                bottom_section,
            ]
            .width(200)
            .height(Length::Fill),
        )
        .style(theme::container_style::sidebar)
        .height(Length::Fill)
        .into()
    }

    fn onboarding_view(&self) -> Element<Message> {
        match self.onboarding.step {
            OnboardingStep::Welcome => self.onboarding_welcome(),
            OnboardingStep::ImportOrCreate => self.onboarding_choice(),
            OnboardingStep::ImportSeed => self.onboarding_import(),
            OnboardingStep::CreateSeed => self.onboarding_create(),
            OnboardingStep::ConfirmSeed => self.onboarding_confirm(),
            OnboardingStep::Complete => self.onboarding_complete(),
        }
    }

    fn onboarding_welcome(&self) -> Element<Message> {
        column![
            text("Ikki").size(48.0),
            text("Private money for everyone"),
            button(text("Get Started")).on_press(Message::StartOnboarding),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn onboarding_choice(&self) -> Element<Message> {
        column![
            text("Welcome to Ikki"),
            text("How would you like to set up your wallet?"),
            button(text("Create new wallet")).on_press(Message::OnboardingCreateNew),
            button(text("Import existing wallet")).on_press(Message::OnboardingImportSeed),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn onboarding_import(&self) -> Element<Message> {
        let error_text: Element<Message> = if let Some(error) = &self.onboarding.error {
            text(error).into()
        } else {
            Space::with_height(0).into()
        };

        column![
            text("Import your wallet"),
            text("Enter your 24-word seed phrase"),
            iced::widget::text_input("Enter your seed phrase...", &self.onboarding.seed_input)
                .on_input(Message::SeedInputChanged)
                .on_submit(Message::SaveSeed),
            error_text,
            row![
                button(text("Back")).on_press(Message::StartOnboarding),
                button(text("Import")).on_press(Message::SaveSeed),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn onboarding_create(&self) -> Element<Message> {
        let seed_display: Element<Message> = match &self.onboarding.generated_seed {
            Some(seed) => text(seed.clone()).into(),
            None => text("Generating...").into(),
        };

        column![
            text("Your seed phrase"),
            text("Write down these 24 words and keep them safe"),
            seed_display,
            text("Never share your seed phrase!"),
            row![
                button(text("Back")).on_press(Message::StartOnboarding),
                button(text("I've saved it")).on_press(Message::ConfirmSeedBackup),
            ]
            .spacing(10),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn onboarding_confirm(&self) -> Element<Message> {
        column![
            text("Confirm your backup"),
            text("Please verify you've saved your seed phrase"),
            button(text("Continue")).on_press(Message::ConfirmSeedBackup),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn onboarding_complete(&self) -> Element<Message> {
        column![
            text("You're all set!"),
            text("Your wallet is ready to use"),
            button(text("Open Wallet")).on_press(Message::OnboardingComplete),
        ]
        .spacing(20)
        .padding(50)
        .into()
    }

    fn load_settings(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let settings_path = config_dir.join("ikki").join("settings.json");
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(url) = settings.get("lightwalletd_url").and_then(|v| v.as_str()) {
                        self.lightwalletd_url = url.to_string();
                    }
                }
            }
        }
    }

    fn save_settings(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let ikki_dir = config_dir.join("ikki");
            let _ = std::fs::create_dir_all(&ikki_dir);
            let settings_path = ikki_dir.join("settings.json");
            let settings = serde_json::json!({
                "lightwalletd_url": self.lightwalletd_url,
                "network": self.network.name(),
            });
            let _ = std::fs::write(
                &settings_path,
                serde_json::to_string_pretty(&settings).unwrap(),
            );
        }
    }
}

// Async operations

async fn initial_load(config: ZcashConfig) -> Result<(u64, String, u64)> {
    use zots_zcash::ZotsWallet;

    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    wallet.sync().await?;

    let block_height = wallet.get_block_height().await.unwrap_or(0);
    let balance = wallet.get_balance().unwrap_or(0);
    let address = wallet.get_address().unwrap_or_default();

    Ok((balance, address, block_height))
}

async fn sync_wallet(config: ZcashConfig) -> Result<(u64, u64)> {
    use zots_zcash::ZotsWallet;

    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    wallet.sync().await?;

    let block_height = wallet.get_block_height().await.unwrap_or(0);
    let balance = wallet.get_balance().unwrap_or(0);

    Ok((block_height, balance))
}

async fn send_transaction(
    config: ZcashConfig,
    address: String,
    amount: u64,
    memo: String,
) -> Result<(String, u64)> {
    use zots_zcash::ZotsWallet;

    let mut wallet = ZotsWallet::new(config).await?;
    wallet.init_account().await?;
    wallet.sync().await?;

    let memo_bytes = if memo.is_empty() {
        None
    } else {
        Some(memo.as_bytes().to_vec())
    };

    let result = wallet.send_to_address(&address, amount, memo_bytes).await?;

    Ok((result.txid, result.fee))
}

async fn load_transactions(_config: ZcashConfig) -> Result<Vec<Transaction>> {
    // In a full implementation, this would query the wallet for transaction history
    // For now, return empty list
    Ok(Vec::new())
}

async fn generate_seed() -> Result<String> {
    use bip0039::{Count, English, Mnemonic};
    let mnemonic = Mnemonic::<English>::generate(Count::Words24);
    Ok(mnemonic.phrase().to_string())
}

/// Run the Ikki application
pub fn run() -> iced::Result {
    // Install rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Load environment variables
    let _ = dotenvy::dotenv();

    iced::application(IkkiApp::title, IkkiApp::update, IkkiApp::view)
        .subscription(IkkiApp::subscription)
        .theme(IkkiApp::theme)
        .window_size((1100.0, 750.0))
        .antialiasing(true)
        .run_with(IkkiApp::new)
}
