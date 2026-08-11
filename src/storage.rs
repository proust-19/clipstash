use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClipboardHistory {
    pub entries: Vec<ClipboardEntry>,
    max_entries: usize,
}

impl ClipboardHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let data = fs::read_to_string(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let history: ClipboardHistory = serde_json::from_str(&data)
                .with_context(|| "Failed to parse clipboard history")?;
            Ok(history)
        } else {
            Ok(Self::new(100))
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let data = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize clipboard history")?;

        // Atomic write: write to temp file then rename
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, &data)
            .with_context(|| format!("Failed to write to {}", temp_path.display()))?;
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to rename {} to {}", temp_path.display(), path.display()))?;

        Ok(())
    }

    pub fn add_entry(&mut self, content: String) -> bool {
        // Skip empty content
        if content.trim().is_empty() {
            return false;
        }

        // Skip duplicates (check last entry)
        if let Some(last) = self.entries.last() {
            if last.content == content {
                return false;
            }
        }

        let id = self.entries.len() as u64 + 1;
        let entry = ClipboardEntry {
            id,
            content,
            timestamp: Utc::now(),
            pinned: false,
        };

        self.entries.push(entry);

        // Remove unpinned entries if over limit
        let unpinned_count = self.entries.iter().filter(|e| !e.pinned).count();
        if unpinned_count > self.max_entries {
            let to_remove = unpinned_count - self.max_entries;
            let mut removed = 0;
            self.entries.retain(|e| {
                if !e.pinned && removed < to_remove {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }

        true
    }

    pub fn get_entry(&self, id: u64) -> Option<&ClipboardEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn clear(&mut self) {
        self.entries.retain(|e| e.pinned);
    }

    pub fn list(&self) -> &[ClipboardEntry] {
        &self.entries
    }

    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.content.to_lowercase().contains(&query_lower))
            .collect()
    }
}
