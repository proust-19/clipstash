use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::clipboard::ClipboardMonitor;
use crate::storage::ClipboardHistory;

#[derive(Parser)]
#[command(name = "clipstash")]
#[command(about = "A fast, keyboard-first clipboard history manager for Wayland/Linux")]
#[command(version)]
pub struct Cli {
    /// Path to the clipboard history file
    #[arg(long, default_value = "~/.local/share/clipstash/history.json")]
    pub history_file: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the clipboard monitoring daemon
    Daemon {
        /// Maximum number of clipboard entries to keep
        #[arg(short, long, default_value = "100")]
        max_entries: usize,
    },

    /// List clipboard history
    List {
        /// Search query to filter entries
        #[arg(short, long)]
        search: Option<String>,

        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Copy a clipboard entry to clipboard
    Select {
        /// ID of the entry to copy
        id: u64,
    },

    /// Clear clipboard history
    Clear {
        /// Keep pinned entries
        #[arg(short, long, default_value = "true")]
        keep_pinned: bool,
    },

    /// Show the most recent clipboard entry
    Latest,

    /// Show clipboard status and statistics
    Status,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let history_path = expand_path(&cli.history_file.unwrap_or_else(|| {
        "~/.local/share/clipstash/history.json".to_string()
    }))?;

    // Ensure the directory exists
    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    match cli.command {
        Commands::Daemon { max_entries } => {
            run_daemon(&history_path, max_entries)?;
        }
        Commands::List { search, limit } => {
            run_list(&history_path, search, limit)?;
        }
        Commands::Select { id } => {
            run_select(&history_path, id)?;
        }
        Commands::Clear { keep_pinned } => {
            run_clear(&history_path, keep_pinned)?;
        }
        Commands::Latest => {
            run_latest(&history_path)?;
        }
        Commands::Status => {
            run_status(&history_path)?;
        }
    }

    Ok(())
}

fn run_daemon(history_path: &PathBuf, _max_entries: usize) -> Result<()> {
    println!("Starting ClipStash daemon...");
    println!("Monitoring clipboard for changes...");
    println!("Press Ctrl+C to stop.");

    let mut history = ClipboardHistory::load(history_path)
        .with_context(|| "Failed to load clipboard history")?;
    let mut monitor = ClipboardMonitor::new();

    // Get initial clipboard content
    if let Ok(Some(content)) = monitor.get_current_content() {
        if history.add_entry(content) {
            history.save(history_path)?;
            println!("Captured initial clipboard content.");
        }
    }

    // Monitor clipboard in a loop
    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));

        if monitor.has_changed() {
            if let Some(content) = monitor.last_content() {
                let content = content.to_string();
                if history.add_entry(content.clone()) {
                    history.save(history_path)?;
                    println!("[{}] Captured: {}", chrono::Local::now().format("%H:%M:%S"), truncate(&content, 50));
                }
            }
        }
    }
}

fn run_list(history_path: &PathBuf, search: Option<String>, limit: usize) -> Result<()> {
    let history = ClipboardHistory::load(history_path)?;

    let entries = if let Some(query) = &search {
        history.search(query)
    } else {
        history.list().iter().rev().take(limit).collect()
    };

    if entries.is_empty() {
        println!("No clipboard entries found.");
        return Ok(());
    }

    println!("Clipboard History ({} entries):", entries.len());
    println!("{:-<60}", "");

    for entry in entries.iter().take(limit) {
        let pinned = if entry.pinned { " [PINNED]" } else { "" };
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
        let content = truncate(&entry.content, 50);
        println!("{:>4}. [{}] {}{}", entry.id, timestamp, content, pinned);
    }

    Ok(())
}

fn run_select(history_path: &PathBuf, id: u64) -> Result<()> {
    let history = ClipboardHistory::load(history_path)?;

    if let Some(entry) = history.get_entry(id) {
        let monitor = ClipboardMonitor::new();
        monitor.set_content(&entry.content)?;
        println!("Copied entry {} to clipboard.", id);
        println!("Content: {}", truncate(&entry.content, 50));
    } else {
        eprintln!("Error: Entry with ID {} not found.", id);
        std::process::exit(1);
    }

    Ok(())
}

fn run_clear(history_path: &PathBuf, keep_pinned: bool) -> Result<()> {
    let mut history = ClipboardHistory::load(history_path)?;

    let before = history.list().len();
    if keep_pinned {
        history.clear();
    } else {
        history = ClipboardHistory::new(100);
    }
    let after = history.list().len();

    history.save(history_path)?;
    println!("Cleared {} entries ({} remaining).", before - after, after);

    Ok(())
}

fn run_latest(history_path: &PathBuf) -> Result<()> {
    let history = ClipboardHistory::load(history_path)?;

    if let Some(entry) = history.list().last() {
        println!("{}", entry.content);
    } else {
        eprintln!("No clipboard entries found.");
        std::process::exit(1);
    }

    Ok(())
}

fn run_status(history_path: &PathBuf) -> Result<()> {
    let history = ClipboardHistory::load(history_path)?;

    println!("ClipStash Status");
    println!("{:-<40}", "");
    println!("History file: {}", history_path.display());
    println!("Total entries: {}", history.list().len());
    println!("Pinned entries: {}", history.list().iter().filter(|e| e.pinned).count());

    if let Some(entry) = history.list().last() {
        println!("Last copied: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S"));
        println!("Last content: {}", truncate(&entry.content, 50));
    }

    Ok(())
}

fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
