use std::time::Instant;

use anyhow::Result;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tokio::time::{sleep, Duration};
use tracing::instrument;

const PASTE_SETTLE_DELAY_MS: u64 = 120;

pub struct ClipboardInjector;

impl ClipboardInjector {
    pub fn new() -> Self {
        Self
    }

    #[instrument(skip(self, transcript), fields(transcript_len = transcript.len()))]
    pub async fn inject_text(&self, transcript: &str) -> Result<()> {
        let start = Instant::now();
        let mut clipboard = Clipboard::new()?;
        let cached_text = clipboard.get_text().ok();

        clipboard.set_text(transcript.to_owned())?;

        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('v'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;

        // Give the target application a moment to read clipboard contents before restoring.
        sleep(Duration::from_millis(PASTE_SETTLE_DELAY_MS)).await;

        match cached_text {
            Some(text) => {
                let _ = clipboard.set_text(text);
            }
            None => {
                let _ = clipboard.clear();
            }
        }
        
        log::info!(
            "clipboard injection completed in {}ms",
            start.elapsed().as_millis()
        );
        Ok(())
    }
}
