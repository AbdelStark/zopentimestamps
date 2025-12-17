//! Main application state and update logic

use crate::message::{HistoryEntry, Message, StampPhase, StampResult, VerifyResult, View};
use crate::theme;
use crate::views;
use anyhow::Result;
use iced::widget::{button, column, container, horizontal_space, row, text, Space};
use iced::{Element, Font, Length, Subscription, Task};
use std::path::PathBuf;
use std::time::Duration;
use zots_core::HashAlgorithm;
use zots_zcash::ZcashConfig;

/// Main application state
pub struct ZotsApp {
    // Navigation
    pub current_view: View,

    // Wallet state
    pub config: Option<ZcashConfig>,
    pub seed_input: String,
    pub block_height: u64,
    pub balance: u64,
    pub wallet_syncing: bool,
    pub wallet_error: Option<String>,

    // Stamp state
    pub stamp_input: String,
    pub stamp_file: Option<PathBuf>,
    pub hash_algorithm: HashAlgorithm,
    pub stamp_phase: StampPhase,
    pub stamp_result: Option<StampResult>,
    pub stamp_error: Option<String>,

    // Verify state
    pub verify_file_input: String,
    pub verify_proof_input: String,
    pub verify_file: Option<PathBuf>,
    pub verify_proof: Option<PathBuf>,
    pub verify_result: Option<VerifyResult>,
    pub verify_error: Option<String>,
    pub verifying: bool,

    // History state
    pub history: Vec<HistoryEntry>,
    pub history_loading: bool,

    // Settings state
    pub explorer_url: String,
    pub lightwalletd_url: String,
    pub settings_saved: bool,

    // UI state
    pub spinner_frame: usize,
    pub copied_feedback: bool,
    pub status_message: String,
}

impl Default for ZotsApp {
    fn default() -> Self {
        Self {
            current_view: View::Home,
            config: None,
            seed_input: String::new(),
            block_height: 0,
            balance: 0,
            wallet_syncing: false,
            wallet_error: None,
            stamp_input: String::new(),
            stamp_file: None,
            hash_algorithm: HashAlgorithm::Sha256,
            stamp_phase: StampPhase::Idle,
            stamp_result: None,
            stamp_error: None,
            verify_file_input: String::new(),
            verify_proof_input: String::new(),
            verify_file: None,
            verify_proof: None,
            verify_result: None,
            verify_error: None,
            verifying: false,
            history: Vec::new(),
            history_loading: false,
            explorer_url: "https://blockexplorer.one/zcash/testnet".to_string(),
            lightwalletd_url: "https://zcash.mysideoftheweb.com:19067".to_string(),
            settings_saved: false,
            spinner_frame: 0,
            copied_feedback: false,
            status_message: "Ready".to_string(),
        }
    }
}

impl ZotsApp {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        // Try to load config from environment
        if let Ok(config) = ZcashConfig::from_env() {
            app.config = Some(config.clone());
            app.status_message = "Syncing wallet...".to_string();
            app.wallet_syncing = true;

            // Start initial sync
            let task = Task::perform(initial_sync(config), |result| match result {
                Ok((height, balance)) => Message::InitialSyncComplete {
                    block_height: height,
                    balance,
                },
                Err(_) => Message::InitialSyncFailed,
            });

            // Load settings
            app.load_settings();

            return (app, task);
        }

        app.status_message = "No wallet configured".to_string();
        app.load_settings();

        (app, Task::none())
    }

    fn title(&self) -> String {
        format!("zOpenTimestamps - {}", self.current_view.title())
    }

    fn theme(&self) -> iced::Theme {
        theme::dark_theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Tick for spinner animation during async operations
        if self.is_busy() {
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
                self.clear_results();
                if view == View::History {
                    return Task::perform(async {}, |_| Message::LoadHistory);
                }
                Task::none()
            }

            // Wallet
            Message::SeedInputChanged(seed) => {
                self.seed_input = seed;
                Task::none()
            }
            Message::SaveSeed => {
                if let Ok(config) = ZcashConfig::from_seed(&self.seed_input) {
                    self.config = Some(config.clone());
                    self.seed_input.clear();
                    self.status_message = "Seed saved, syncing...".to_string();
                    self.wallet_syncing = true;
                    return Task::perform(sync_wallet(config), |result| match result {
                        Ok((height, balance)) => Message::WalletSynced {
                            block_height: height,
                            balance,
                        },
                        Err(e) => Message::WalletSyncFailed(e.to_string()),
                    });
                } else {
                    self.wallet_error = Some("Invalid seed phrase".to_string());
                }
                Task::none()
            }
            Message::SyncWallet => {
                if let Some(config) = &self.config {
                    self.wallet_syncing = true;
                    self.wallet_error = None;
                    self.status_message = "Syncing wallet...".to_string();
                    return Task::perform(sync_wallet(config.clone()), |result| match result {
                        Ok((height, balance)) => Message::WalletSynced {
                            block_height: height,
                            balance,
                        },
                        Err(e) => Message::WalletSyncFailed(e.to_string()),
                    });
                }
                Task::none()
            }
            Message::WalletSynced {
                block_height,
                balance,
            } => {
                self.block_height = block_height;
                self.balance = balance;
                self.wallet_syncing = false;
                self.status_message = "Synced".to_string();
                Task::none()
            }
            Message::WalletSyncFailed(error) => {
                self.wallet_error = Some(error);
                self.wallet_syncing = false;
                self.status_message = "Sync failed".to_string();
                Task::none()
            }
            Message::InitialSyncComplete {
                block_height,
                balance,
            } => {
                self.block_height = block_height;
                self.balance = balance;
                self.wallet_syncing = false;
                self.status_message = "Ready".to_string();
                Task::none()
            }
            Message::InitialSyncFailed => {
                self.wallet_syncing = false;
                self.status_message = "Ready (sync failed)".to_string();
                Task::none()
            }

            // Stamp
            Message::StampInputChanged(input) => {
                self.stamp_input = input;
                Task::none()
            }
            Message::SelectFile => Task::perform(pick_file(), Message::FileSelected),
            Message::FileSelected(path) => {
                if let Some(p) = path {
                    self.stamp_input = p.display().to_string();
                    self.stamp_file = Some(p);
                }
                Task::none()
            }
            Message::ToggleAlgorithm => {
                self.hash_algorithm = match self.hash_algorithm {
                    HashAlgorithm::Sha256 => HashAlgorithm::Blake3,
                    HashAlgorithm::Blake3 => HashAlgorithm::Sha256,
                };
                Task::none()
            }
            Message::StartStamp => {
                if self.stamp_input.is_empty() {
                    self.stamp_error = Some("Please enter a file path or hash".to_string());
                    return Task::none();
                }
                if self.config.is_none() {
                    self.stamp_error = Some("No wallet configured".to_string());
                    return Task::none();
                }

                self.stamp_phase = StampPhase::Syncing;
                self.stamp_error = None;
                self.stamp_result = None;
                self.status_message = "Creating timestamp...".to_string();

                let config = self.config.clone().unwrap();
                let input = self.stamp_input.clone();
                let algorithm = self.hash_algorithm;

                Task::perform(run_stamp(config, input, algorithm), |result| match result {
                    Ok(stamp_result) => Message::StampComplete(stamp_result),
                    Err(e) => Message::StampFailed(e.to_string()),
                })
            }
            Message::StampProgress(phase) => {
                self.stamp_phase = phase;
                Task::none()
            }
            Message::StampComplete(result) => {
                self.stamp_result = Some(result);
                self.stamp_phase = StampPhase::Complete;
                self.status_message = "Timestamp created!".to_string();
                Task::none()
            }
            Message::StampFailed(error) => {
                self.stamp_error = Some(error);
                self.stamp_phase = StampPhase::Failed;
                self.status_message = "Stamp failed".to_string();
                Task::none()
            }

            // Verify
            Message::VerifyFileInputChanged(input) => {
                self.verify_file_input = input;
                Task::none()
            }
            Message::VerifyProofInputChanged(input) => {
                self.verify_proof_input = input;
                Task::none()
            }
            Message::SelectVerifyFile => Task::perform(pick_file(), Message::VerifyFileSelected),
            Message::SelectProofFile => Task::perform(pick_proof_file(), Message::ProofFileSelected),
            Message::VerifyFileSelected(path) => {
                if let Some(p) = path {
                    self.verify_file_input = p.display().to_string();
                    self.verify_file = Some(p);
                }
                Task::none()
            }
            Message::ProofFileSelected(path) => {
                if let Some(p) = path {
                    self.verify_proof_input = p.display().to_string();
                    self.verify_proof = Some(p);
                }
                Task::none()
            }
            Message::StartVerify => {
                if self.verify_proof_input.is_empty() {
                    self.verify_error = Some("Please select a proof file".to_string());
                    return Task::none();
                }

                self.verifying = true;
                self.verify_error = None;
                self.verify_result = None;
                self.status_message = "Verifying...".to_string();

                let file_input = self.verify_file_input.clone();
                let proof_path = PathBuf::from(&self.verify_proof_input);
                let config = self.config.clone();

                Task::perform(
                    run_verify(config, file_input, proof_path),
                    |result| match result {
                        Ok(verify_result) => Message::VerifyComplete(verify_result),
                        Err(e) => Message::VerifyFailed(e.to_string()),
                    },
                )
            }
            Message::VerifyComplete(result) => {
                self.verify_result = Some(result);
                self.verifying = false;
                self.status_message = "Verification complete".to_string();
                Task::none()
            }
            Message::VerifyFailed(error) => {
                self.verify_error = Some(error);
                self.verifying = false;
                self.status_message = "Verification failed".to_string();
                Task::none()
            }

            // History
            Message::LoadHistory => {
                self.history_loading = true;
                Task::perform(load_history(), Message::HistoryLoaded)
            }
            Message::HistoryLoaded(entries) => {
                self.history = entries;
                self.history_loading = false;
                Task::none()
            }
            Message::DeleteProof(path) => Task::perform(delete_proof(path), |result| {
                match result {
                    Ok(path) => Message::ProofDeleted(path),
                    Err(_) => Message::LoadHistory, // Reload on error
                }
            }),
            Message::ProofDeleted(path) => {
                self.history.retain(|e| e.path != path);
                Task::none()
            }

            // Settings
            Message::ExplorerUrlChanged(url) => {
                self.explorer_url = url;
                self.settings_saved = false;
                Task::none()
            }
            Message::LightwalletdUrlChanged(url) => {
                self.lightwalletd_url = url;
                self.settings_saved = false;
                Task::none()
            }
            Message::SaveSettings => {
                self.save_settings();
                self.settings_saved = true;
                Task::none()
            }
            Message::ResetSettings => {
                self.explorer_url = "https://blockexplorer.one/zcash/testnet".to_string();
                self.lightwalletd_url = "https://zcash.mysideoftheweb.com:19067".to_string();
                self.settings_saved = false;
                Task::none()
            }

            // UI
            Message::Tick => {
                self.spinner_frame = (self.spinner_frame + 1) % 8;
                Task::none()
            }
            Message::CopyToClipboard(text) => {
                self.copied_feedback = true;
                Task::perform(copy_to_clipboard(text), |_| Message::Copied)
            }
            Message::Copied => {
                // Reset feedback after delay
                Task::perform(
                    async {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    },
                    |_| Message::DismissResult,
                )
            }
            Message::OpenExplorer(url) => {
                let _ = open::that(&url);
                Task::none()
            }
            Message::DismissResult => {
                self.copied_feedback = false;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let sidebar = self.sidebar();
        let content = match self.current_view {
            View::Home => views::home::view(self),
            View::Stamp => views::stamp::view(self),
            View::Verify => views::verify::view(self),
            View::History => views::history::view(self),
            View::Wallet => views::wallet::view(self),
            View::Settings => views::settings::view(self),
        };
        let status_bar = self.status_bar();

        let main_content = column![
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(24),
            status_bar,
        ];

        row![sidebar, main_content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn sidebar(&self) -> Element<Message> {
        let nav_items = [
            View::Home,
            View::Stamp,
            View::Verify,
            View::History,
            View::Wallet,
            View::Settings,
        ];

        let nav_buttons: Vec<Element<Message>> = nav_items
            .iter()
            .map(|view| {
                let is_active = self.current_view == *view;
                button(
                    row![
                        text(view.icon()).size(18),
                        Space::with_width(12),
                        text(view.title()).size(14),
                    ]
                    .padding([8, 12])
                    .align_y(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .style(move |theme, status| theme::button_style::nav(theme, status, is_active))
                .on_press(Message::NavigateTo(*view))
                .into()
            })
            .collect();

        let logo = row![
            text("⏰").size(28),
            Space::with_width(8),
            column![
                text("zOpenTimestamps").size(16).font(Font::DEFAULT),
                text("Zcash Timestamping")
                    .size(11)
                    .style(theme::text_style::muted()),
            ],
        ]
        .align_y(iced::Alignment::Center)
        .padding([16, 16]);

        let nav = column(nav_buttons).spacing(4).padding([8, 12]);

        container(
            column![logo, Space::with_height(8), nav]
                .width(200)
                .height(Length::Fill),
        )
        .style(theme::container_style::sidebar)
        .height(Length::Fill)
        .into()
    }

    fn status_bar(&self) -> Element<Message> {
        let spinner = if self.is_busy() {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
            text(frames[self.spinner_frame]).style(theme::text_style::primary())
        } else {
            text("●").style(theme::text_style::accent())
        };

        let network = text(format!(
            "{}  Block: {}",
            self.config
                .as_ref()
                .map(|c| c.network.name())
                .unwrap_or("No wallet"),
            self.block_height
        ))
        .size(12)
        .style(theme::text_style::muted());

        let balance_zec = self.balance as f64 / 100_000_000.0;
        let balance = text(format!("{balance_zec:.8} ZEC"))
            .size(12)
            .style(theme::text_style::accent());

        let status = text(&self.status_message)
            .size(12)
            .style(theme::text_style::muted());

        container(
            row![
                spinner,
                Space::with_width(12),
                status,
                horizontal_space(),
                network,
                Space::with_width(24),
                balance,
            ]
            .align_y(iced::Alignment::Center)
            .padding([8, 16]),
        )
        .style(theme::container_style::status_bar)
        .width(Length::Fill)
        .into()
    }

    fn is_busy(&self) -> bool {
        self.wallet_syncing || self.stamp_phase.is_busy() || self.verifying || self.history_loading
    }

    fn clear_results(&mut self) {
        self.stamp_error = None;
        self.stamp_result = None;
        self.stamp_phase = StampPhase::Idle;
        self.verify_error = None;
        self.verify_result = None;
    }

    fn load_settings(&mut self) {
        if let Some(config_dir) = dirs::config_dir() {
            let settings_path = config_dir.join("zots").join("settings.json");
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(url) = settings.get("explorer_url").and_then(|v| v.as_str()) {
                        self.explorer_url = url.to_string();
                    }
                    if let Some(url) = settings.get("lightwalletd_url").and_then(|v| v.as_str()) {
                        self.lightwalletd_url = url.to_string();
                    }
                }
            }
        }
    }

    fn save_settings(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let zots_dir = config_dir.join("zots");
            let _ = std::fs::create_dir_all(&zots_dir);
            let settings_path = zots_dir.join("settings.json");
            let settings = serde_json::json!({
                "explorer_url": self.explorer_url,
                "lightwalletd_url": self.lightwalletd_url,
            });
            let _ = std::fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap());
        }
    }

    pub fn spinner(&self) -> &'static str {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        frames[self.spinner_frame]
    }
}

// Async operations

async fn initial_sync(config: ZcashConfig) -> Result<(u64, u64)> {
    sync_wallet(config).await
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

async fn run_stamp(
    config: ZcashConfig,
    input: String,
    algorithm: HashAlgorithm,
) -> Result<StampResult> {
    use std::path::Path;
    use zots_core::{
        hash_file_with, hash_from_hex_with, hash_to_hex, TimestampProof, ZcashAttestation,
    };
    use zots_zcash::ZotsWallet;

    // Compute hash
    let path = Path::new(&input);
    let (hash_bytes, output_path) = if path.exists() {
        let h = hash_file_with(path, algorithm)?;
        let output = PathBuf::from(format!(
            "{}.zots",
            path.file_name().unwrap_or_default().to_string_lossy()
        ));
        (h, output)
    } else if input.len() >= 40 {
        let h = hash_from_hex_with(&input, algorithm)?;
        let output = PathBuf::from(format!("{}.zots", &input[..16]));
        (h, output)
    } else {
        anyhow::bail!("File not found and input is not a valid hash");
    };

    let hash_hex = hash_to_hex(&hash_bytes);

    // Create wallet and sync
    let mut wallet = ZotsWallet::new(config.clone()).await?;
    wallet.init_account().await?;
    wallet.sync().await?;

    // Create and broadcast transaction
    let tx_result = wallet.create_timestamp_tx(&hash_bytes).await?;
    let txid = tx_result.txid.clone();

    // Wait for confirmation
    let confirmation = wallet.wait_confirmation(&txid, 10).await?;

    // Create proof
    let network = config.network;
    let mut proof = TimestampProof::new_with_algorithm(hash_bytes, algorithm);
    proof.add_attestation(ZcashAttestation::new(
        network,
        tx_result.txid_bytes,
        confirmation.block_height,
        confirmation.block_time,
        0,
    ));

    // Save proof
    proof.save(&output_path)?;

    let compact = proof.to_compact().unwrap_or_default();
    let explorer_link = proof.attestations[0].explorer_link();

    Ok(StampResult {
        hash: hash_hex,
        algorithm,
        txid,
        block_height: confirmation.block_height,
        block_time: confirmation.block_time as u64,
        output_path,
        compact,
        explorer_link,
    })
}

async fn run_verify(
    config: Option<ZcashConfig>,
    file_input: String,
    proof_path: PathBuf,
) -> Result<VerifyResult> {
    use zots_core::{hash_file_with, hash_from_hex_with, TimestampProof};
    use zots_zcash::ZotsWallet;

    // Load proof
    let proof = TimestampProof::load(&proof_path)?;
    let proof_hash_bytes = proof.hash_bytes()?;
    let algorithm = proof.hash_algorithm();
    let compact = proof.to_compact().unwrap_or_default();

    // Check file/hash match
    let file_hash_matches = if !file_input.is_empty() {
        let path = std::path::Path::new(&file_input);
        let recomputed = if path.exists() {
            hash_file_with(path, algorithm)?
        } else {
            hash_from_hex_with(&file_input, algorithm)?
        };
        Some(recomputed == proof_hash_bytes)
    } else {
        None
    };

    if let Some(false) = file_hash_matches {
        return Ok(VerifyResult {
            hash: proof.hash.clone(),
            algorithm,
            compact,
            valid: false,
            network: String::new(),
            block_height: 0,
            timestamp: String::new(),
            txid: String::new(),
            explorer_link: String::new(),
            error: Some("Hash does NOT match the provided file/hash!".to_string()),
            file_hash_matches: Some(false),
        });
    }

    if proof.attestations.is_empty() {
        return Ok(VerifyResult {
            hash: proof.hash.clone(),
            algorithm,
            compact,
            valid: false,
            network: String::new(),
            block_height: 0,
            timestamp: String::new(),
            txid: String::new(),
            explorer_link: String::new(),
            error: Some("Proof is pending (no attestations)".to_string()),
            file_hash_matches,
        });
    }

    let att = &proof.attestations[0];

    // Verify on blockchain if wallet available
    if let Some(cfg) = config {
        let mut wallet = ZotsWallet::new(cfg).await?;
        wallet.init_account().await?;

        let txid_bytes = att.txid_bytes()?;
        let vr = wallet
            .verify_timestamp_tx(&txid_bytes, &proof_hash_bytes, Some(att.block_height))
            .await?;

        Ok(VerifyResult {
            hash: proof.hash.clone(),
            algorithm,
            compact,
            valid: vr.valid,
            network: att.network.to_string(),
            block_height: att.block_height,
            timestamp: att.timestamp().to_rfc3339(),
            txid: att.txid_hex().to_string(),
            explorer_link: att.explorer_link(),
            error: vr.error,
            file_hash_matches,
        })
    } else {
        // No wallet, just show proof info
        Ok(VerifyResult {
            hash: proof.hash.clone(),
            algorithm,
            compact,
            valid: true,
            network: att.network.to_string(),
            block_height: att.block_height,
            timestamp: att.timestamp().to_rfc3339(),
            txid: att.txid_hex().to_string(),
            explorer_link: att.explorer_link(),
            error: Some("Cannot verify on-chain (no wallet)".to_string()),
            file_hash_matches,
        })
    }
}

async fn load_history() -> Vec<HistoryEntry> {
    use zots_core::TimestampProof;

    let mut entries = Vec::new();

    // Look for .zots files in current directory
    if let Ok(read_dir) = std::fs::read_dir(".") {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zots") {
                if let Ok(proof) = TimestampProof::load(&path) {
                    let confirmed = !proof.attestations.is_empty();
                    let (network, block_height) = if confirmed {
                        let att = &proof.attestations[0];
                        (Some(att.network.to_string()), Some(att.block_height))
                    } else {
                        (None, None)
                    };

                    entries.push(HistoryEntry {
                        path: path.clone(),
                        hash: proof.hash.clone(),
                        algorithm: proof.hash_algorithm(),
                        created: entry
                            .metadata()
                            .ok()
                            .and_then(|m| m.created().ok())
                            .map(|t| {
                                chrono::DateTime::<chrono::Local>::from(t)
                                    .format("%Y-%m-%d %H:%M")
                                    .to_string()
                            })
                            .unwrap_or_else(|| "Unknown".to_string()),
                        confirmed,
                        network,
                        block_height,
                    });
                }
            }
        }
    }

    entries
}

async fn delete_proof(path: PathBuf) -> Result<PathBuf> {
    std::fs::remove_file(&path)?;
    Ok(path)
}

async fn pick_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Select file to timestamp")
        .pick_file()
        .await
        .map(|f| f.path().to_path_buf())
}

async fn pick_proof_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Select proof file")
        .add_filter("zots", &["zots"])
        .add_filter("JSON", &["json"])
        .pick_file()
        .await
        .map(|f| f.path().to_path_buf())
}

async fn copy_to_clipboard(_text: String) -> Result<()> {
    // Clipboard is handled by iced's clipboard feature
    Ok(())
}

/// Run the application
pub fn run() -> iced::Result {
    // Install rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Load environment variables
    let _ = dotenvy::dotenv();

    iced::application(ZotsApp::title, ZotsApp::update, ZotsApp::view)
        .subscription(ZotsApp::subscription)
        .theme(ZotsApp::theme)
        .window_size((1200.0, 800.0))
        .antialiasing(true)
        .run_with(ZotsApp::new)
}
