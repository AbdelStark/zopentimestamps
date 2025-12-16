//! TUI rendering
//!
//! Renders the terminal user interface with:
//! - ASCII art header with cypherpunk aesthetic
//! - Dynamic content based on current screen/state
//! - Progress indicators for async operations
//! - Detailed result displays for stamp and verify operations

use chrono::{DateTime, Utc};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, AppState, OperationPhase, VerifyStep};

/// ASCII art header for cypherpunk aesthetic
const ASCII_HEADER: &str = r#"
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║   ███████╗ ██████╗ ████████╗███████╗                             ║
║   ╚══███╔╝██╔═══██╗╚══██╔══╝██╔════╝                             ║
║     ███╔╝ ██║   ██║   ██║   ███████╗                             ║
║    ███╔╝  ██║   ██║   ██║   ╚════██║                             ║
║   ███████╗╚██████╔╝   ██║   ███████║                             ║
║   ╚══════╝ ╚═════╝    ╚═╝   ╚══════╝                             ║
║                                                                   ║
║          z O P E N T I M E S T A M P S                           ║
║          Zcash Blockchain Timestamping                           ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
"#;

/// Main draw function
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(14), // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(ASCII_HEADER).style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // Main content based on state
    match app.state {
        AppState::Menu => draw_menu(f, chunks[1]),
        AppState::Stamp => draw_stamp(f, chunks[1], app),
        AppState::Verify => draw_verify(f, chunks[1], app),
        AppState::Wallet => draw_wallet(f, chunks[1], app),
    }

    // Status bar
    draw_status_bar(f, chunks[2], app);
}

fn draw_menu(f: &mut Frame, area: Rect) {
    let menu_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [S] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Stamp a file or hash"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [V] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Verify a timestamp proof"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [W] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Wallet management"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Q] ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit"),
        ]),
    ];

    let menu = Paragraph::new(menu_text)
        .block(Block::default().borders(Borders::ALL).title("Main Menu"))
        .wrap(Wrap { trim: false });
    f.render_widget(menu, area);
}

fn draw_stamp(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![];

    match &app.operation_phase {
        OperationPhase::Input | OperationPhase::Failed => {
            content.push(Line::from(vec![
                Span::styled("Enter file path or hash to timestamp:", Style::default().fg(Color::White)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::raw(&app.input_buffer),
                Span::styled("█", Style::default().fg(Color::Gray)),
            ]));
            content.push(Line::from(""));

            if !app.result_message.is_empty() {
                // Split result message by newlines for proper display
                for line in app.result_message.lines() {
                    content.push(Line::from(Span::styled(
                        line,
                        Style::default().fg(if app.result_is_error {
                            Color::Red
                        } else {
                            Color::Green
                        }),
                    )));
                }
                content.push(Line::from(""));
            }

            content.push(Line::from(Span::styled(
                "[ESC] Back to menu  [ENTER] Submit",
                Style::default().fg(Color::Gray),
            )));
        }
        OperationPhase::Hashing => {
            content.push(Line::from(vec![
                Span::styled(app.spinner(), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled("Hashing file...", Style::default().fg(Color::Yellow)),
            ]));
        }
        OperationPhase::Syncing => {
            content.push(Line::from(vec![
                Span::styled(app.spinner(), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled("Syncing wallet with blockchain...", Style::default().fg(Color::Yellow)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "This may take a moment for initial sync",
                Style::default().fg(Color::Gray),
            )));
        }
        OperationPhase::Broadcasting => {
            content.push(Line::from(vec![
                Span::styled(app.spinner(), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled("Creating and broadcasting transaction...", Style::default().fg(Color::Cyan)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "Building zk-SNARK proof and sending to network",
                Style::default().fg(Color::Gray),
            )));
        }
        OperationPhase::WaitingConfirmation { txid, .. } => {
            content.push(Line::from(vec![
                Span::styled(app.spinner(), Style::default().fg(Color::Magenta)),
                Span::raw(" "),
                Span::styled("Waiting for block confirmation...", Style::default().fg(Color::Magenta)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(vec![
                Span::styled("TXID: ", Style::default().fg(Color::Gray)),
                Span::styled(&txid[..24], Style::default().fg(Color::Yellow)),
                Span::styled("...", Style::default().fg(Color::Gray)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "Transaction broadcast - waiting for next block (~75 seconds)",
                Style::default().fg(Color::Gray),
            )));
        }
        OperationPhase::Complete => {
            if let Some(ref result) = app.stamp_result {
                // Success header
                content.push(Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("TIMESTAMP CONFIRMED", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));
                content.push(Line::from(""));

                // Details section
                content.push(Line::from(vec![
                    Span::styled("  Hash:   ", Style::default().fg(Color::Gray)),
                    Span::styled(&result.hash[..32], Style::default().fg(Color::White)),
                    Span::styled("...", Style::default().fg(Color::Gray)),
                ]));
                content.push(Line::from(vec![
                    Span::styled("  TXID:   ", Style::default().fg(Color::Gray)),
                    Span::styled(&result.txid[..32], Style::default().fg(Color::Yellow)),
                    Span::styled("...", Style::default().fg(Color::Gray)),
                ]));
                content.push(Line::from(vec![
                    Span::styled("  Block:  ", Style::default().fg(Color::Gray)),
                    Span::styled(result.block_height.to_string(), Style::default().fg(Color::Cyan)),
                ]));
                // Format block time as human-readable
                let time_str = DateTime::<Utc>::from_timestamp(result.block_time as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| "Unknown".to_string());
                content.push(Line::from(vec![
                    Span::styled("  Time:   ", Style::default().fg(Color::Gray)),
                    Span::styled(time_str, Style::default().fg(Color::Magenta)),
                ]));
                content.push(Line::from(vec![
                    Span::styled("  Saved:  ", Style::default().fg(Color::Gray)),
                    Span::styled(&result.output_path, Style::default().fg(Color::Green)),
                ]));
                content.push(Line::from(""));

                // Compact format section
                content.push(Line::from(Span::styled(
                    "  Embeddable proof (copy this):",
                    Style::default().fg(Color::Gray),
                )));
                content.push(Line::from(""));
                // Show compact format (truncated if too long)
                let compact_display = if result.compact.len() > 60 {
                    format!("  {}...", &result.compact[..60])
                } else {
                    format!("  {}", result.compact)
                };
                content.push(Line::from(Span::styled(
                    compact_display,
                    Style::default().fg(Color::Magenta),
                )));
                content.push(Line::from(""));
                content.push(Line::from(vec![
                    Span::styled("  Length: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{} chars", result.compact.len()), Style::default().fg(Color::White)),
                ]));
            }
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "[ESC] Back to menu",
                Style::default().fg(Color::Gray),
            )));
        }
    }

    let stamp = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Stamp "))
        .wrap(Wrap { trim: false });
    f.render_widget(stamp, area);
}

fn draw_verify(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![];

    match app.verify_step {
        VerifyStep::FileOrHash => {
            // Step 1: Enter file or hash
            content.push(Line::from(vec![
                Span::styled("Step 1/2: ", Style::default().fg(Color::Cyan)),
                Span::styled("Enter file path or hash to verify:", Style::default().fg(Color::White)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::raw(&app.input_buffer),
                Span::styled("█", Style::default().fg(Color::Gray)),
            ]));
            content.push(Line::from(""));

            if !app.result_message.is_empty() {
                content.push(Line::from(Span::styled(
                    &app.result_message,
                    Style::default().fg(if app.result_is_error {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                )));
                content.push(Line::from(""));
            }

            content.push(Line::from(Span::styled(
                "[ESC] Back to menu  [ENTER] Continue",
                Style::default().fg(Color::Gray),
            )));
        }
        VerifyStep::ProofPath => {
            // Step 2: Enter proof path
            content.push(Line::from(vec![
                Span::styled("Step 2/2: ", Style::default().fg(Color::Cyan)),
                Span::styled("Enter proof file path (.zots):", Style::default().fg(Color::White)),
            ]));
            content.push(Line::from(""));

            // Show what was entered in step 1
            content.push(Line::from(vec![
                Span::styled("  Verifying: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if app.verify_file_input.len() > 40 {
                        format!("{}...", &app.verify_file_input[..40])
                    } else {
                        app.verify_file_input.clone()
                    },
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::raw(&app.input_buffer),
                Span::styled("█", Style::default().fg(Color::Gray)),
            ]));
            content.push(Line::from(""));

            if !app.result_message.is_empty() {
                content.push(Line::from(Span::styled(
                    &app.result_message,
                    Style::default().fg(if app.result_is_error {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                )));
                content.push(Line::from(""));
            }

            content.push(Line::from(Span::styled(
                "[ESC] Back to menu  [ENTER] Verify",
                Style::default().fg(Color::Gray),
            )));
        }
        VerifyStep::Verifying => {
            // Verifying in progress
            content.push(Line::from(vec![
                Span::styled(app.spinner(), Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled("Verifying against blockchain...", Style::default().fg(Color::Cyan)),
            ]));
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "Fetching transaction and decrypting memo",
                Style::default().fg(Color::Gray),
            )));
        }
        VerifyStep::Results => {
            // Show verification results
            if let Some(ref result) = app.verify_result {
                if result.valid {
                    // Valid timestamp
                    content.push(Line::from(vec![
                        Span::styled("✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled("VALID TIMESTAMP", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(" (verified on-chain)", Style::default().fg(Color::Gray)),
                    ]));
                } else {
                    // Invalid or error
                    content.push(Line::from(vec![
                        Span::styled("✗ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::styled("VERIFICATION FAILED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    ]));
                }
                content.push(Line::from(""));

                // Show file hash match status
                if let Some(matches) = result.file_hash_matches {
                    content.push(Line::from(vec![
                        Span::styled("  File Match: ", Style::default().fg(Color::Gray)),
                        if matches {
                            Span::styled("✓ Hash matches", Style::default().fg(Color::Green))
                        } else {
                            Span::styled("✗ Hash does NOT match!", Style::default().fg(Color::Red))
                        },
                    ]));
                }

                // Show details
                if !result.hash.is_empty() {
                    content.push(Line::from(vec![
                        Span::styled("  Hash:     ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            if result.hash.len() > 32 {
                                format!("{}...", &result.hash[..32])
                            } else {
                                result.hash.clone()
                            },
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }

                if !result.network.is_empty() {
                    content.push(Line::from(vec![
                        Span::styled("  Network:  ", Style::default().fg(Color::Gray)),
                        Span::styled(&result.network, Style::default().fg(Color::Yellow)),
                    ]));
                }

                if result.block_height > 0 {
                    content.push(Line::from(vec![
                        Span::styled("  Block:    ", Style::default().fg(Color::Gray)),
                        Span::styled(result.block_height.to_string(), Style::default().fg(Color::Cyan)),
                    ]));
                }

                if !result.timestamp.is_empty() {
                    content.push(Line::from(vec![
                        Span::styled("  Time:     ", Style::default().fg(Color::Gray)),
                        Span::styled(&result.timestamp, Style::default().fg(Color::Magenta)),
                    ]));
                }

                if !result.txid.is_empty() {
                    content.push(Line::from(vec![
                        Span::styled("  TXID:     ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            if result.txid.len() > 32 {
                                format!("{}...", &result.txid[..32])
                            } else {
                                result.txid.clone()
                            },
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }

                // Show error if any
                if let Some(ref error) = result.error {
                    content.push(Line::from(""));
                    content.push(Line::from(vec![
                        Span::styled("  Note: ", Style::default().fg(Color::Gray)),
                        Span::styled(error, Style::default().fg(if result.valid { Color::Gray } else { Color::Red })),
                    ]));
                }

                // Show explorer link
                if result.valid && !result.explorer_link.is_empty() {
                    content.push(Line::from(""));
                    content.push(Line::from(vec![
                        Span::styled("  Explorer: ", Style::default().fg(Color::Gray)),
                        Span::styled(&result.explorer_link, Style::default().fg(Color::Blue)),
                    ]));
                }
            }

            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                "[ESC] Back to menu",
                Style::default().fg(Color::Gray),
            )));
        }
    }

    let verify = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Verify "))
        .wrap(Wrap { trim: false });
    f.render_widget(verify, area);
}

fn draw_wallet(f: &mut Frame, area: Rect, app: &App) {
    let balance_zec = app.balance as f64 / 100_000_000.0;
    let mut content = vec![];

    // Check if syncing
    if matches!(app.operation_phase, OperationPhase::Syncing) {
        content.push(Line::from(vec![
            Span::styled(app.spinner(), Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled("Syncing wallet with blockchain...", Style::default().fg(Color::Yellow)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "Scanning blocks for transactions",
            Style::default().fg(Color::Gray),
        )));
    } else {
        // Wallet info section
        content.push(Line::from(vec![
            Span::styled("  Network:      ", Style::default().fg(Color::Gray)),
            Span::styled(app.network_name(), Style::default().fg(Color::Yellow)),
        ]));
        content.push(Line::from(vec![
            Span::styled("  Block Height: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.block_height.to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        content.push(Line::from(vec![
            Span::styled("  Balance:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.8} TAZ", balance_zec),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]));
        content.push(Line::from(""));

        if !app.result_message.is_empty() {
            content.push(Line::from(Span::styled(
                format!("  {}", &app.result_message),
                Style::default().fg(if app.result_is_error {
                    Color::Red
                } else {
                    Color::Green
                }),
            )));
            content.push(Line::from(""));
        }

        // Commands section
        content.push(Line::from(Span::styled(
            "  Commands:",
            Style::default().fg(Color::White),
        )));
        content.push(Line::from(vec![
            Span::styled("    [S] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("Sync wallet", Style::default().fg(Color::Gray)),
        ]));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "  [ESC] Back to menu",
            Style::default().fg(Color::Gray),
        )));
    }

    let wallet = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(" Wallet "))
        .wrap(Wrap { trim: false });
    f.render_widget(wallet, area);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let balance_zec = app.balance as f64 / 100_000_000.0;

    let status = Paragraph::new(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(&app.status_message, Style::default().fg(Color::Green)),
        Span::raw(" │ "),
        Span::styled("Block: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.block_height.to_string(),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" │ "),
        Span::styled("Balance: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.8} TAZ", balance_zec),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" │ "),
        Span::styled("Network: ", Style::default().fg(Color::Gray)),
        Span::styled(app.network_name(), Style::default().fg(Color::Cyan)),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, area);
}
