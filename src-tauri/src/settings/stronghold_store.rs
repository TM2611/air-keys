use anyhow::{Context, Result};
use async_trait::async_trait;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

const CLIENT_ID: &[u8] = b"air_keys_client";
const RECORD_KEY: &str = "deepgram_api_key";

#[async_trait]
pub trait SecureKeyStore: Send + Sync {
    async fn save_deepgram_key(&self, api_key: String) -> Result<()>;
    async fn read_deepgram_key(&self) -> Result<Option<String>>;
    async fn clear_deepgram_key(&self) -> Result<()>;
}

pub struct StrongholdStore {
    stronghold: Mutex<tauri_plugin_stronghold::stronghold::Stronghold>,
}

impl StrongholdStore {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data = app_handle
            .path()
            .app_local_data_dir()
            .context("could not resolve local data directory")?;
        std::fs::create_dir_all(&app_data).context("could not create local data directory")?;

        let salt_path = app_data.join("air-keys-vault-salt.bin");
        let snapshot_path = app_data.join("air-keys.vault.hold");
        let password = Self::build_vault_password(app_handle);
        let key = tauri_plugin_stronghold::kdf::KeyDerivation::argon2(&password, &salt_path);
        let stronghold = tauri_plugin_stronghold::stronghold::Stronghold::new(snapshot_path, key)
            .context("could not initialise stronghold vault")?;

        Ok(Self {
            stronghold: Mutex::new(stronghold),
        })
    }

    fn build_vault_password(app_handle: &AppHandle) -> String {
        let user = std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "unknown-user".to_string());
        let host = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unknown-host".to_string());
        format!("{user}:{host}:{}", app_handle.config().identifier)
    }
}

#[async_trait]
impl SecureKeyStore for StrongholdStore {
    async fn save_deepgram_key(&self, api_key: String) -> Result<()> {
        let stronghold = self.stronghold.lock().await;
        if stronghold.get_client(CLIENT_ID.to_vec()).is_err() {
            stronghold.create_client(CLIENT_ID.to_vec())?;
        }
        let client = stronghold.get_client(CLIENT_ID.to_vec())?;
        client.store().insert(
            RECORD_KEY.as_bytes().to_vec(),
            api_key.as_bytes().to_vec(),
            None,
        )?;
        stronghold.save()?;
        Ok(())
    }

    async fn read_deepgram_key(&self) -> Result<Option<String>> {
        let stronghold = self.stronghold.lock().await;
        let client = match stronghold.get_client(CLIENT_ID.to_vec()) {
            Ok(client) => client,
            Err(_) => return Ok(None),
        };
        let value = client.store().get(RECORD_KEY.as_bytes())?;
        let Some(value) = value else {
            return Ok(None);
        };
        let parsed = String::from_utf8(value).context("stored key is not valid utf-8")?;
        Ok(Some(parsed))
    }

    async fn clear_deepgram_key(&self) -> Result<()> {
        let stronghold = self.stronghold.lock().await;
        if let Ok(client) = stronghold.get_client(CLIENT_ID.to_vec()) {
            let _ = client.store().delete(RECORD_KEY.as_bytes());
            stronghold.save()?;
        }
        Ok(())
    }
}
