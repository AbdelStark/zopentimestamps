//! Colored CLI output helpers

use colored::*;
use qrcode::{QrCode, render::unicode};

/// Print a header with underline
pub fn print_header(text: &str) {
    println!();
    println!("{}", text.cyan().bold());
    println!("{}", "─".repeat(text.len()).cyan());
}

/// Print a success message with checkmark
pub fn print_success(text: &str) {
    println!("{} {}", "✓".green().bold(), text.green());
}

/// Print an error message with X
pub fn print_error(text: &str) {
    println!("{} {}", "✗".red().bold(), text.red());
}

/// Print a warning message
pub fn print_warning(text: &str) {
    println!("{} {}", "!".yellow().bold(), text.yellow());
}

/// Print a labeled info line
pub fn print_info(label: &str, value: &str) {
    println!("  {}: {}", label.white().bold(), value);
}

/// Print a hash in yellow with its algorithm
pub fn print_hash(hash: &str, algorithm: &str) {
    println!(
        "  {} ({}): {}",
        "Hash".white().bold(),
        algorithm,
        hash.yellow()
    );
}

/// Print a clickable terminal hyperlink
pub fn print_link(label: &str, url: &str) {
    // OSC 8 terminal hyperlink escape sequence
    println!(
        "  {}: \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        label,
        url,
        url.blue().underline()
    );
}

/// Print a status line (for progress updates)
pub fn print_status(text: &str) {
    println!("  {} {}", "→".cyan(), text);
}

/// Render a QR code as a string for terminal display
pub fn render_qr(data: &str) -> anyhow::Result<String> {
    let code = QrCode::new(data.as_bytes())?;
    let rendered = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    Ok(rendered)
}

/// Print a QR code with a label
pub fn print_qr(label: &str, data: &str) -> anyhow::Result<()> {
    println!();
    print_header(label);
    let qr = render_qr(data)?;
    println!("{qr}");
    Ok(())
}
