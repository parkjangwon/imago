use crate::error::{ImagoError, Result};
use base64::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_TIMEOUT: u64 = 120;
const MODEL_FALLBACKS: [&str; 4] = [
    "gemini-2.5-flash-image",
    "gemini-3.1-flash-image-preview",
    "gemini-3-pro-image-preview",
    "gemini-2.0-flash-exp-image-generation",
];

/// Gemini API client
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

/// Request payload for content generation
#[derive(Debug, Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "responseModalities")]
    response_modalities: Vec<String>,
}

/// Response from content generation
#[derive(Debug, Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
    #[serde(rename = "promptFeedback")]
    prompt_feedback: Option<PromptFeedback>,
    #[allow(dead_code)]
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponsePart {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineData,
    },
}

#[derive(Debug, Deserialize)]
struct InlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct SafetyRating {
    category: String,
    probability: String,
    blocked: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PromptFeedback {
    #[allow(dead_code)]
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<SafetyRating>>,
    #[serde(rename = "blockReason")]
    block_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
    #[allow(dead_code)]
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u64>,
    #[allow(dead_code)]
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u64>,
    #[allow(dead_code)]
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u64>,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            model,
        }
    }

    /// Generate an image from a text prompt
    pub async fn generate_image(&self, prompt: &str) -> Result<(Vec<u8>, Option<String>)> {
        let request = GenerateContentRequest {
            contents: vec![Content {
                parts: vec![Part::Text {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                response_modalities: vec!["IMAGE".to_string()],
            },
        };

        let response = self.send_request(&request).await?;
        self.extract_image_data(response)
    }

    /// Send the API request
    async fn send_request(
        &self,
        request: &GenerateContentRequest,
    ) -> Result<GenerateContentResponse> {
        let mut tried = Vec::new();

        for model in std::iter::once(self.model.as_str()).chain(MODEL_FALLBACKS.iter().copied()) {
            if tried.contains(&model.to_string()) {
                continue;
            }
            tried.push(model.to_string());

            let url = format!(
                "{}/{}:generateContent?key={}",
                API_BASE_URL, model, self.api_key
            );

            let response = self.client.post(&url).json(request).send().await?;
            let status = response.status();

            if status.is_success() {
                let response_text = response.text().await?;
                let parsed: GenerateContentResponse = serde_json::from_str(&response_text)
                    .map_err(|e| ImagoError::ResponseFormatError {
                        message: format!("Failed to parse API response: {}", e),
                    })?;
                return Ok(parsed);
            }

            let error_text = response.text().await.unwrap_or_default();
            if status.as_u16() != 404 {
                return Err(ImagoError::ApiError {
                    status: status.as_u16(),
                    message: error_text,
                });
            }
        }

        Err(ImagoError::ApiResponseError(format!(
            "No available image model found. Tried: {}",
            tried.join(", ")
        )))
    }

    /// Extract image data from API response
    fn extract_image_data(
        &self,
        response: GenerateContentResponse,
    ) -> Result<(Vec<u8>, Option<String>)> {
        // Check for prompt feedback (blocks, etc.)
        if let Some(feedback) = response.prompt_feedback {
            if let Some(reason) = feedback.block_reason {
                return Err(ImagoError::SafetyFilter(format!(
                    "Request blocked: {}",
                    reason
                )));
            }
        }

        // Get candidates
        let candidates = response.candidates.ok_or_else(|| ImagoError::NoImageData)?;

        let candidate = candidates
            .into_iter()
            .next()
            .ok_or(ImagoError::NoImageData)?;

        // Check finish reason
        if let Some(reason) = candidate.finish_reason {
            if reason != "STOP" {
                // Check safety ratings for more info
                if let Some(ratings) = candidate.safety_ratings {
                    let blocked: Vec<_> = ratings
                        .iter()
                        .filter(|r| r.blocked.unwrap_or(false))
                        .map(|r| format!("{}: {}", r.category, r.probability))
                        .collect();

                    if !blocked.is_empty() {
                        return Err(ImagoError::SafetyFilter(blocked.join(", ")));
                    }
                }

                if reason == "IMAGE_SAFETY" {
                    return Err(ImagoError::SafetyFilter(
                        "Image content blocked by safety filters".to_string(),
                    ));
                }
            }
        }

        // Extract content
        let content = candidate.content.ok_or(ImagoError::NoImageData)?;
        let mut text_response = None;

        // Find the inline data part
        for part in content.parts {
            match part {
                ResponsePart::InlineData { inline_data } => {
                    if inline_data.mime_type.starts_with("image/") {
                        let image_bytes = BASE64_STANDARD.decode(&inline_data.data)?;
                        return Ok((image_bytes, text_response));
                    }
                }
                ResponsePart::Text { text } => {
                    text_response = Some(text);
                }
            }
        }

        // If we got here and have text but no image, the model probably returned a message
        if let Some(text) = text_response {
            return Err(ImagoError::ApiResponseError(format!(
                "Model returned text instead of image: {}",
                text
            )));
        }

        Err(ImagoError::NoImageData)
    }
}
