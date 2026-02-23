use anyhow::Result;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub struct ClipboardInjector;

impl ClipboardInjector {
    pub fn new() -> Self {
        Self
    }

    pub async fn inject_text(&self, transcript: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        let cached_text = clipboard.get_text().ok();

        clipboard.set_text(transcript.to_owned())?;

        let mut enigo = Enigo::new(&Settings::default())?;
        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('v'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;

        match cached_text {
            Some(text) => {
                let _ = clipboard.set_text(text);
            }
            None => {
                let _ = clipboard.clear();
            }
        }
        Ok(())
    }
}
