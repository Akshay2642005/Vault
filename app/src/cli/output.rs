// Output formatting utilities - currently unused but ready for implementation
// TODO: Integrate with CLI commands for consistent output formatting

use owo_colors::OwoColorize;
use crate::storage::SecretMetadata;

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

pub fn print_secret_list(secrets: &[(String, SecretMetadata)], detailed: bool) {
    if detailed {
        for (key, meta) in secrets {
            println!("  {} {}", "•".green(), key.cyan());
            println!("    Created: {}", meta.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("    Version: {}", meta.version);
            if !meta.tags.is_empty() {
                println!("    Tags: {}", meta.tags.join(", ").yellow());
            }
        }
    } else {
        for (key, _) in secrets {
            println!("  {}", key.cyan());
        }
    }
}

pub fn print_table_header(columns: &[&str]) {
    println!("{}", columns.join(" | ").bold());
    println!("{}", "-".repeat(columns.join(" | ").len()));
}