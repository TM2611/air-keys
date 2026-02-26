use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use tracing::instrument;

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
    #[instrument(skip(self, audio_path))]
    async fn process_file(&self, audio_path: &Path) -> Result<String, AudioProcessorError> {
        let total_start = Instant::now();
        
        let api_key = self
            .key_store
            .read_deepgram_key()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?
            .ok_or(AudioProcessorError::MissingApiKey)?;

        let read_start = Instant::now();
        let audio_bytes = std::fs::read(audio_path).map_err(|err| {
            AudioProcessorError::Request(format!("could not read audio file: {err}"))
        })?;
        let file_size = audio_bytes.len();
        let read_duration = read_start.elapsed();

        let api_start = Instant::now();
        let response = self
            .client
            .post(DEEPGRAM_ENDPOINT)
            .header(AUTHORIZATION, format!("Token {api_key}"))
            .header(CONTENT_TYPE, "audio/wav")
            .body(audio_bytes)
            .send()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?;
        let api_duration = api_start.elapsed();

        if !response.status().is_success() {
            return Err(AudioProcessorError::Request(format!(
                "deepgram returned status {}",
                response.status()
            )));
        }

        let parse_start = Instant::now();
        let payload: DeepgramResponse = response.json().await.map_err(|err| {
            AudioProcessorError::Request(format!("invalid deepgram payload: {err}"))
        })?;
        let parse_duration = parse_start.elapsed();

        let transcript = payload
            .results
            .channels
            .into_iter()
            .flat_map(|channel| channel.alternatives.into_iter())
            .map(|alt| alt.transcript)
            .find(|transcript| !transcript.trim().is_empty())
            .ok_or(AudioProcessorError::EmptyTranscript)?;

        let total_duration = total_start.elapsed();
        log::info!(
            "deepgram transcription completed total={}ms read={}ms api={}ms parse={}ms file_size={}B transcript_len={}",
            total_duration.as_millis(),
            read_duration.as_millis(),
            api_duration.as_millis(),
            parse_duration.as_millis(),
            file_size,
            transcript.len()
        );

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
