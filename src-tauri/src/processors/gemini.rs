use std::sync::Arc;

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde_json::json;

use crate::core::audio_processor::{AudioProcessorError, TranscriptCleaner};
use crate::settings::stronghold_store::SecureKeyStore;

const GEMINI_ENDPOINT: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-3-flash-preview:generateContent";

#[derive(Clone)]
pub struct GeminiCleaner {
    client: reqwest::Client,
    key_store: Arc<dyn SecureKeyStore>,
}

impl GeminiCleaner {
    pub fn new(key_store: Arc<dyn SecureKeyStore>) -> Self {
        Self {
            client: reqwest::Client::new(),
            key_store,
        }
    }
}

#[async_trait]
impl TranscriptCleaner for GeminiCleaner {
    async fn clean(&self, transcript: &str) -> Result<String, AudioProcessorError> {
        if transcript.trim().is_empty() {
            return Ok(String::new());
        }

        let api_key = self
            .key_store
            .read_gemini_key()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?
            .ok_or(AudioProcessorError::MissingGeminiApiKey)?;

        let prompt = format!(
            "You are a dictation cleanup assistant.\n\
             Rewrite the transcript so it reads naturally while preserving meaning.\n\
             Remove filler words (for example: um, uh, ah, er, like, you know), fix grammar,\n\
             punctuation, and sentence flow.\n\
             Return only the cleaned transcript with no explanation.\n\n\
             Transcript:\n{}",
            transcript
        );

        let response = self
            .client
            .post(GEMINI_ENDPOINT)
            .header("x-goog-api-key", api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "contents": [
                    {
                        "parts": [
                            {
                                "text": prompt
                            }
                        ]
                    }
                ]
            }))
            .send()
            .await
            .map_err(|err| AudioProcessorError::Request(err.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AudioProcessorError::Request(format!(
                "gemini returned status {status}: {body}"
            )));
        }

        let payload: GeminiResponse = response
            .json()
            .await
            .map_err(|err| AudioProcessorError::Request(format!("invalid gemini payload: {err}")))?;

        let cleaned = payload
            .candidates
            .into_iter()
            .flat_map(|candidate| candidate.content.parts.into_iter())
            .map(|part| part.text)
            .find(|text| !text.trim().is_empty())
            .ok_or_else(|| {
                AudioProcessorError::Request("gemini returned an empty transcript".to_string())
            })?;

        Ok(cleaned.trim().to_string())
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: String,
}
