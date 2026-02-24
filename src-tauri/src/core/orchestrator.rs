use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::audio::recorder::Recorder;
use crate::core::audio_processor::{AudioProcessor, AudioProcessorError};
use crate::injection::clipboard_injector::ClipboardInjector;

const TRAY_ID: &str = "air_keys_tray";
const RECORDING_WINDOW_ID: &str = "recording";
const RECORDING_AMPLITUDE_EVENT: &str = "recording-amplitude";
const AMPLITUDE_POLL_MS: u64 = 50;
/// Offset from bottom of screen (above taskbar/toolbar) in logical pixels.
const RECORDING_BOTTOM_OFFSET: i32 = 72;
const MIN_RECORDING_DURATION: Duration = Duration::from_millis(500);

#[derive(Clone, Serialize)]
struct RecordingAmplitudePayload {
    level: f32,
}

pub struct DictationOrchestrator {
    app_handle: AppHandle,
    recorder: Mutex<Recorder>,
    processor: Arc<dyn AudioProcessor>,
    injector: ClipboardInjector,
    recording_path: Mutex<Option<PathBuf>>,
    recording_started_at: Mutex<Option<Instant>>,
    amplitude_level: Arc<AtomicU32>,
    level_emitter_task: Mutex<Option<JoinHandle<()>>>,
}

impl DictationOrchestrator {
    pub fn new(app_handle: AppHandle, processor: Arc<dyn AudioProcessor>) -> Result<Self> {
        Ok(Self {
            app_handle,
            recorder: Mutex::new(Recorder::new()?),
            processor,
            injector: ClipboardInjector::new(),
            recording_path: Mutex::new(None),
            recording_started_at: Mutex::new(None),
            amplitude_level: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            level_emitter_task: Mutex::new(None),
        })
    }

    pub async fn handle_alt_double_tap(&self) -> Result<()> {
        let mut recorder = self.recorder.lock().await;
        if recorder.is_recording() {
            recorder.stop()?;
            self.set_tray_recording(false);
            self.set_recording_window_visible(false);
            self.stop_level_emitter().await;
            let maybe_path = self.recording_path.lock().await.take();
            let started_at = self.recording_started_at.lock().await.take();
            drop(recorder);

            if let Some(path) = maybe_path {
                if let Some(started_at) = started_at {
                    if started_at.elapsed() < MIN_RECORDING_DURATION {
                        let _ = std::fs::remove_file(&path);
                        log::info!(
                            "discarded short recording (< {}ms)",
                            MIN_RECORDING_DURATION.as_millis()
                        );
                        return Ok(());
                    }
                }
                self.transcribe_and_inject(path).await?;
            }
            return Ok(());
        }

        let temp_path = std::env::temp_dir().join(format!(
            "air-keys-{}.wav",
            Utc::now().format("%Y%m%d-%H%M%S")
        ));
        self.amplitude_level.store(0.0f32.to_bits(), Ordering::Relaxed);
        recorder
            .start(temp_path.clone(), Some(self.amplitude_level.clone()))
            .context("failed to start recording")?;
        *self.recording_path.lock().await = Some(temp_path);
        *self.recording_started_at.lock().await = Some(Instant::now());
        self.set_tray_recording(true);
        self.set_recording_window_visible(true);
        self.start_level_emitter().await;
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

    fn set_recording_window_visible(&self, is_visible: bool) {
        let Some(window) = self.app_handle.get_webview_window(RECORDING_WINDOW_ID) else {
            return;
        };
        if is_visible {
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let mon_pos = monitor.position();
                let mon_size = monitor.size();
                if let Ok(win_size) = window.inner_size() {
                    let x = mon_pos.x + (mon_size.width as i32 - win_size.width as i32) / 2;
                    let y = mon_pos.y
                        + (mon_size.height as i32 - win_size.height as i32)
                        - RECORDING_BOTTOM_OFFSET;
                    let _ = window.set_position(PhysicalPosition::new(x, y));
                }
            }
            let _ = window.show();
        } else {
            let _ = window.hide();
        }
    }

    async fn start_level_emitter(&self) {
        self.stop_level_emitter().await;
        let app_handle = self.app_handle.clone();
        let amplitude_level = self.amplitude_level.clone();

        let handle = tokio::spawn(async move {
            loop {
                if let Some(window) = app_handle.get_webview_window(RECORDING_WINDOW_ID) {
                    let level = f32::from_bits(amplitude_level.load(Ordering::Relaxed));
                    let _ = window.emit(
                        RECORDING_AMPLITUDE_EVENT,
                        RecordingAmplitudePayload { level },
                    );
                }
                tokio::time::sleep(Duration::from_millis(AMPLITUDE_POLL_MS)).await;
            }
        });
        *self.level_emitter_task.lock().await = Some(handle);
    }

    async fn stop_level_emitter(&self) {
        if let Some(handle) = self.level_emitter_task.lock().await.take() {
            handle.abort();
        }
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
