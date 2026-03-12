//! Output formatting for terminal and JSON output.

use colored::Colorize;
use tabled::{Table, Tabled};

/// Row for the social sets table.
#[derive(Debug, Clone, Tabled)]
pub struct SocialSetRow {
    /// Social set ID.
    #[tabled(rename = "ID")]
    pub id: String,
    /// Display name.
    #[tabled(rename = "Name")]
    pub name: String,
    /// Connected platforms.
    #[tabled(rename = "Platforms")]
    pub platforms: String,
}

/// Row for the drafts table.
#[derive(Debug, Clone, Tabled)]
pub struct DraftRow {
    /// Draft ID.
    #[tabled(rename = "ID")]
    pub id: String,
    /// Current status.
    #[tabled(rename = "Status")]
    pub status: String,
    /// Truncated content preview.
    #[tabled(rename = "Content")]
    pub content: String,
    /// Scheduled date (if any).
    #[tabled(rename = "Scheduled")]
    pub scheduled: String,
}

/// Row for the tags table.
#[derive(Debug, Clone, Tabled)]
pub struct TagRow {
    /// Tag ID.
    #[tabled(rename = "ID")]
    pub id: String,
    /// Tag name.
    #[tabled(rename = "Name")]
    pub name: String,
}

/// Print a success message to stderr.
pub fn print_success(msg: &str) {
    eprintln!("{} {msg}", "Success!".green());
}

/// Print an error message to stderr.
pub fn print_error(msg: &str) {
    eprintln!("{} {msg}", "Error:".red().bold());
}

/// Print a table of rows, or a fallback message if empty.
pub fn print_table<T: Tabled>(rows: &[T], empty_msg: &str) {
    if rows.is_empty() {
        println!("{empty_msg}");
    } else {
        println!("{}", Table::new(rows));
    }
}

/// Print JSON to stdout.
pub fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_string())
    );
}

/// Truncate a string to a maximum length, appending "..." if truncated.
#[must_use]
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let end = max.saturating_sub(3);
        // Find a valid char boundary
        let end = s.floor_char_boundary(end);
        format!("{}...", &s[..end])
    }
}

/// Extract a string field from a JSON value, returning an empty string on miss.
pub fn json_str(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_string()
}
