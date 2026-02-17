use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// --- Styles ---

pub fn success_style() -> Style {
    Style::new().green().bold()
}

pub fn error_style() -> Style {
    Style::new().red().bold()
}

pub fn info_style() -> Style {
    Style::new().cyan()
}

pub fn dim_style() -> Style {
    Style::new().dim()
}

pub fn bold_style() -> Style {
    Style::new().bold()
}

pub fn warn_style() -> Style {
    Style::new().yellow()
}

// --- Message helpers ---

/// Print a success message to stdout: "  OK  message"
pub fn success(msg: &str) {
    println!("{} {}", success_style().apply_to("  OK "), msg);
}

/// Print a status/progress message to stderr with a right-aligned label.
pub fn status(label: &str, msg: &str) {
    eprintln!("{} {}", info_style().apply_to(format!("{:>5}", label)), msg);
}

/// Print a warning to stderr.
pub fn warn(msg: &str) {
    eprintln!("{} {}", warn_style().apply_to(" WARN"), msg);
}

/// Print an indented detail line to stderr.
pub fn detail(msg: &str) {
    eprintln!("       {}", dim_style().apply_to(msg));
}

// --- Spinner ---

/// Create and start a spinner on stderr. Returns the `ProgressBar` handle —
/// call `finish_and_clear()` when the operation completes.
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["   ", ".  ", ".. ", "...", " ..", "  .", "   "]),
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(msg.to_string());
    pb
}

// --- Error display ---

/// Print a formatted error to stderr (red "error:" prefix with anyhow chain).
pub fn print_error(err: &anyhow::Error) {
    eprintln!("{} {:#}", error_style().apply_to("error:"), err);
}

// --- Table helpers ---

/// Print a bold header row with a `─` underline.
pub fn table_header(columns: &[(&str, usize)]) {
    let header: String = columns
        .iter()
        .map(|(name, width)| format!("{:<width$}", bold_style().apply_to(name), width = width))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{header}");
    let total_width: usize = columns.iter().map(|(_, w)| w + 2).sum::<usize>();
    println!("{}", dim_style().apply_to("─".repeat(total_width)));
}
