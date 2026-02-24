use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::audio::recorder::Recorder;
use crate::core::audio_processor::{AudioProcessor, AudioProcessorError};
use crate::injection::clipboard_injector::ClipboardInjector;

const TRAY_ID: &str = "air_keys_tray";

pub struct DictationOrchestrator {
    app_handle: AppHandle,
    recorder: Mutex<Recorder>,
    processor: Arc<dyn AudioProcessor>,
    injector: ClipboardInjector,
    recording_path: Mutex<Option<PathBuf>>,
}

impl DictationOrchestrator {
    pub fn new(app_handle: AppHandle, processor: Arc<dyn AudioProcessor>) -> Result<Self> {
        Ok(Self {
            app_handle,
            recorder: Mutex::new(Recorder::new()?),
            processor,
            injector: ClipboardInjector::new(),
            recording_path: Mutex::new(None),
        })
    }

    pub async fn handle_alt_double_tap(&self) -> Result<()> {
        let mut recorder = self.recorder.lock().await;
        if recorder.is_recording() {
            recorder.stop()?;
            self.set_tray_recording(false);
            let maybe_path = self.recording_path.lock().await.take();
            drop(recorder);

            if let Some(path) = maybe_path {
                self.transcribe_and_inject(path).await?;
            }
            return Ok(());
        }

        let temp_path = std::env::temp_dir().join(format!(
            "air-keys-{}.wav",
            Utc::now().format("%Y%m%d-%H%M%S")
        ));
        recorder
            .start(temp_path.clone())
            .context("failed to start recording")?;
        *self.recording_path.lock().await = Some(temp_path);
        self.set_tray_recording(true);
        Ok(())
    }

    fn set_tray_recording(&self, is_recording: bool) {
        let Some(tray) = self.app_handle.tray_by_id(TRAY_ID) else {
            return;
        };
        let tooltip = if is_recording {
            "Air Keys - recording"
        } else {
            "Air Keys - idle"
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    async fn transcribe_and_inject(&self, path: PathBuf) -> Result<()> {
        let result = self.processor.process_file(&path).await;
        let _ = std::fs::remove_file(&path);

        match result {
            Ok(transcript) => {
                self.injector.inject_text(&transcript).await?;
                Ok(())
            }
            Err(AudioProcessorError::EmptyTranscript) => {
                log::warn!("dictation captured but transcript was empty; skipping paste");
                Ok(())
            }
            Err(err) => Err(anyhow::anyhow!("{err}")),
        }
    }
}
