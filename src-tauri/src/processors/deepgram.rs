use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

use crate::core::audio_processor::{AudioProcessor, AudioProcessorError};
use crate::settings::stronghold_store::SecureKeyStore;

const DEEPGRAM_ENDPOINT: &str = "https://api.deepgram.com/v1/listen?model=nova-2&smart_format=true&filler_words=false&punctuate=true";

#[derive(Clone)]
pub struct DeepgramProcessor {
    client: reqwest::Client,
    key_store: Arc<dyn SecureKeyStore>,
}

impl DeepgramProcessor {
    pub fn new(key_store: Arc<dyn SecureKeyStore>) -> Self {
        Self {
            client: reqwest::Client::new(),
            key_store,
        }
    }
}

#[async_trait]
impl AudioProcessor for DeepgramProcessor {
    async fn process_file(&self, audio_path: &Path) -> Result<String, AudioProcessorError> {
        let api_key = self
            .key_store
            .read_deepgram_key()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?
            .ok_or(AudioProcessorError::MissingApiKey)?;

        let audio_bytes = std::fs::read(audio_path).map_err(|err| {
            AudioProcessorError::Request(format!("could not read audio file: {err}"))
        })?;

        let response = self
            .client
            .post(DEEPGRAM_ENDPOINT)
            .header(AUTHORIZATION, format!("Token {api_key}"))
            .header(CONTENT_TYPE, "audio/wav")
            .body(audio_bytes)
            .send()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?;

        if !response.status().is_success() {
            return Err(AudioProcessorError::Request(format!(
                "deepgram returned status {}",
                response.status()
            )));
        }

        let payload: DeepgramResponse = response.json().await.map_err(|err| {
            AudioProcessorError::Request(format!("invalid deepgram payload: {err}"))
        })?;

        let transcript = payload
            .results
            .channels
            .into_iter()
            .flat_map(|channel| channel.alternatives.into_iter())
            .map(|alt| alt.transcript)
            .find(|transcript| !transcript.trim().is_empty())
            .ok_or(AudioProcessorError::EmptyTranscript)?;

        Ok(transcript)
    }
}

#[derive(Debug, Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Debug, Deserialize)]
struct DeepgramResults {
    channels: Vec<DeepgramChannel>,
}

#[derive(Debug, Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Deserialize)]
struct DeepgramAlternative {
    transcript: String,
}
