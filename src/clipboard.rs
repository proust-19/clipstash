use anyhow::{Context, Result};
use arboard::Clipboard;

pub struct ClipboardMonitor {
    last_content: Option<String>,
}

impl ClipboardMonitor {
    pub fn new() -> Self {
        Self { last_content: None }
    }

    pub fn get_current_content(&self) -> Result<Option<String>> {
        let mut clipboard = Clipboard::new().with_context(|| "Failed to initialize clipboard")?;

        match clipboard.get_text() {
            Ok(content) => {
                if content.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(content))
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn set_content(&self, text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new().with_context(|| "Failed to initialize clipboard")?;

        clipboard
            .set_text(text)
            .with_context(|| "Failed to copy to clipboard")?;

        Ok(())
    }

    pub fn has_changed(&mut self) -> bool {
        match self.get_current_content() {
            Ok(Some(content)) => {
                let changed = self.last_content.as_ref() != Some(&content);
                self.last_content = Some(content);
                changed
            }
            Ok(None) => {
                let changed = self.last_content.is_some();
                self.last_content = None;
                changed
            }
            Err(_) => false,
        }
    }

    pub fn last_content(&self) -> Option<&str> {
        self.last_content.as_deref()
    }
}

impl Default for ClipboardMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_monitor_creation() {
        let monitor = ClipboardMonitor::new();
        assert!(monitor.last_content.is_none());
    }
}
