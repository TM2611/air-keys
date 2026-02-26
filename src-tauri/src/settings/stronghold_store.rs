use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const KEY_FILE: &str = "air-keys-credentials.json";

#[async_trait]
pub trait SecureKeyStore: Send + Sync {
    async fn save_deepgram_key(&self, api_key: String) -> Result<()>;
    async fn read_deepgram_key(&self) -> Result<Option<String>>;
    async fn clear_deepgram_key(&self) -> Result<()>;
    async fn save_gemini_key(&self, api_key: String) -> Result<()>;
    async fn read_gemini_key(&self) -> Result<Option<String>>;
    async fn clear_gemini_key(&self) -> Result<()>;
    async fn save_processing_enabled(&self, enabled: bool) -> Result<()>;
    async fn read_processing_enabled(&self) -> Result<bool>;
    async fn save_logging_enabled(&self, enabled: bool) -> Result<()>;
    async fn read_logging_enabled(&self) -> Result<bool>;
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct KeyData {
    deepgram_api_key: Option<String>,
    gemini_api_key: Option<String>,
    processing_enabled: Option<bool>,
    logging_enabled: Option<bool>,
}

pub struct StrongholdStore {
    file_path: PathBuf,
    data: Mutex<KeyData>,
}

impl StrongholdStore {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data = app_handle
            .path()
            .app_local_data_dir()
            .context("could not resolve local data directory")?;
        std::fs::create_dir_all(&app_data).context("could not create local data directory")?;

        let file_path = app_data.join(KEY_FILE);
        let data = if file_path.exists() {
            let contents =
                std::fs::read_to_string(&file_path).context("could not read credentials file")?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            KeyData::default()
        };

        Ok(Self {
            file_path,
            data: Mutex::new(data),
        })
    }

    /// Synchronous read for use during app setup (before the async runtime is available).
    pub fn read_logging_enabled_blocking(&self) -> bool {
        self.data
            .try_lock()
            .map(|data| data.logging_enabled.unwrap_or(false))
            .unwrap_or(false)
    }

    fn persist(file_path: &PathBuf, data: &KeyData) -> Result<()> {
        let contents = serde_json::to_string_pretty(data)
            .context("could not serialise credentials")?;
        std::fs::write(file_path, contents).context("could not write credentials file")?;
        Ok(())
    }
}

#[async_trait]
impl SecureKeyStore for StrongholdStore {
    async fn save_deepgram_key(&self, api_key: String) -> Result<()> {
        let mut data = self.data.lock().await;
        data.deepgram_api_key = Some(api_key);
        Self::persist(&self.file_path, &data)
    }

    async fn read_deepgram_key(&self) -> Result<Option<String>> {
        let data = self.data.lock().await;
        Ok(data.deepgram_api_key.clone())
    }

    async fn clear_deepgram_key(&self) -> Result<()> {
        let mut data = self.data.lock().await;
        data.deepgram_api_key = None;
        Self::persist(&self.file_path, &data)
    }

    async fn save_gemini_key(&self, api_key: String) -> Result<()> {
        let mut data = self.data.lock().await;
        data.gemini_api_key = Some(api_key);
        Self::persist(&self.file_path, &data)
    }

    async fn read_gemini_key(&self) -> Result<Option<String>> {
        let data = self.data.lock().await;
        Ok(data.gemini_api_key.clone())
    }

    async fn clear_gemini_key(&self) -> Result<()> {
        let mut data = self.data.lock().await;
        data.gemini_api_key = None;
        Self::persist(&self.file_path, &data)
    }

    async fn save_processing_enabled(&self, enabled: bool) -> Result<()> {
        let mut data = self.data.lock().await;
        data.processing_enabled = Some(enabled);
        Self::persist(&self.file_path, &data)
    }

    async fn read_processing_enabled(&self) -> Result<bool> {
        let data = self.data.lock().await;
        Ok(data.processing_enabled.unwrap_or(false))
    }

    async fn save_logging_enabled(&self, enabled: bool) -> Result<()> {
        let mut data = self.data.lock().await;
        data.logging_enabled = Some(enabled);
        Self::persist(&self.file_path, &data)
    }

    async fn read_logging_enabled(&self) -> Result<bool> {
        let data = self.data.lock().await;
        Ok(data.logging_enabled.unwrap_or(false))
    }
}
