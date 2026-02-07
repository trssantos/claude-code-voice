use anyhow::{Context, Result};
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;
use tracing::info;

pub struct ClipboardHandler {
    clipboard: Clipboard,
    enigo: Enigo,
}

impl ClipboardHandler {
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
        let enigo = Enigo::new(&Settings::default()).context("Failed to initialize enigo")?;

        Ok(Self { clipboard, enigo })
    }

    pub async fn paste_text(&mut self, text: &str) -> Result<()> {
        if text.is_empty() {
            return Ok(());
        }

        info!("Pasting text: {}", text);

        // Save current clipboard content
        let original_clipboard = self.clipboard.get_text().ok();

        // Set new clipboard content
        self.clipboard
            .set_text(text)
            .context("Failed to set clipboard text")?;

        // Small delay to ensure clipboard is set
        thread::sleep(Duration::from_millis(50));

        // Simulate Ctrl+V (or Cmd+V on macOS)
        #[cfg(target_os = "macos")]
        {
            self.enigo.key(Key::Meta, Direction::Press)?;
            self.enigo.key(Key::Unicode('v'), Direction::Click)?;
            self.enigo.key(Key::Meta, Direction::Release)?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.enigo.key(Key::Control, Direction::Press)?;
            self.enigo.key(Key::Unicode('v'), Direction::Click)?;
            self.enigo.key(Key::Control, Direction::Release)?;
        }

        // Wait a bit for paste to complete
        thread::sleep(Duration::from_millis(100));

        // Restore original clipboard content
        if let Some(original) = original_clipboard {
            if let Err(e) = self.clipboard.set_text(&original) {
                tracing::warn!("Failed to restore clipboard: {}", e);
            }
        }

        Ok(())
    }
}
