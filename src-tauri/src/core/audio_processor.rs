use std::path::Path;

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum AudioProcessorError {
    #[error("missing deepgram api key")]
    MissingApiKey,
    #[error("missing gemini api key")]
    MissingGeminiApiKey,
    #[error("audio processing request failed: {0}")]
    Request(String),
    #[error("transcription response was empty")]
    EmptyTranscript,
}

#[async_trait]
pub trait AudioProcessor: Send + Sync {
    async fn process_file(&self, audio_path: &Path) -> Result<String, AudioProcessorError>;
}

#[async_trait]
pub trait TranscriptCleaner: Send + Sync {
    async fn clean(&self, transcript: &str) -> Result<String, AudioProcessorError>;
}
