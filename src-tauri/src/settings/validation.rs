use reqwest::header::{AUTHORIZATION, HeaderValue};
use reqwest::StatusCode;

const DEEPGRAM_VALIDATE_ENDPOINT: &str = "https://api.deepgram.com/v1/auth/token";
const GEMINI_VALIDATE_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1/models";

pub async fn validate_deepgram_key(api_key: &str) -> Result<(), String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err("Deepgram API key is required.".to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .get(DEEPGRAM_VALIDATE_ENDPOINT)
        .header(AUTHORIZATION, format!("Token {trimmed_key}"))
        .send()
        .await
        .map_err(|err| format!("could not validate Deepgram API key: {err}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err("Invalid Deepgram API key. Enter a valid key and try again.".to_string());
    }

    Err(format!(
        "Deepgram key validation failed with status {}.",
        response.status()
    ))
}

pub async fn validate_gemini_key(api_key: &str) -> Result<(), String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err("Gemini API key is required.".to_string());
    }

    let api_key_header = HeaderValue::from_str(trimmed_key)
        .map_err(|_| "Gemini API key contains invalid characters.".to_string())?;

    let client = reqwest::Client::new();
    let response = client
        .get(GEMINI_VALIDATE_ENDPOINT)
        .header("x-goog-api-key", api_key_header)
        .send()
        .await
        .map_err(|err| format!("could not validate Gemini API key: {err}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err("Invalid Gemini API key. Enter a valid key and try again.".to_string());
    }

    Err(format!(
        "Gemini key validation failed with status {}.",
        response.status()
    ))
}
