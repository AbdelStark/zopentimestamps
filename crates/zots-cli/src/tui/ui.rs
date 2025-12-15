//! TUI rendering

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::app::{App, AppState};

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
    let mut content = vec![
        Line::from("Enter file path or hash to timestamp:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&app.input_buffer),
            Span::styled("█", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];

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
        "[ESC] Back to menu  [ENTER] Submit",
        Style::default().fg(Color::Gray),
    )));

    let stamp = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Stamp"))
        .wrap(Wrap { trim: false });
    f.render_widget(stamp, area);
}

fn draw_verify(f: &mut Frame, area: Rect, app: &App) {
    let mut content = vec![
        Line::from("Enter proof file path (.zots):"),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(Color::Green)),
            Span::raw(&app.input_buffer),
            Span::styled("█", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];

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
        "[ESC] Back to menu  [ENTER] Submit",
        Style::default().fg(Color::Gray),
    )));

    let verify = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Verify"))
        .wrap(Wrap { trim: false });
    f.render_widget(verify, area);
}

fn draw_wallet(f: &mut Frame, area: Rect, app: &App) {
    let balance_zec = app.balance as f64 / 100_000_000.0;

    let mut content = vec![
        Line::from(vec![
            Span::styled("Network: ", Style::default().fg(Color::Gray)),
            Span::styled(app.network_name(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Block Height: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.block_height.to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Balance: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.8} TAZ", balance_zec),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
    ];

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

    content.push(Line::from(vec![
        Span::styled("[S] ", Style::default().fg(Color::Cyan)),
        Span::raw("Sync wallet"),
    ]));
    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "[ESC] Back to menu",
        Style::default().fg(Color::Gray),
    )));

    let wallet = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Wallet"))
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
