use thiserror::Error;

/// Application-specific error types
#[derive(Error, Debug)]
pub enum ImagoError {
    #[error("API key not found. Please set GEMINI_API_KEY environment variable")]
    MissingApiKey,

    #[error("API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("API response error: {0}")]
    ApiResponseError(String),

    #[error("No image data found in response")]
    NoImageData,

    #[error("Safety filter blocked image generation. Reason: {0}")]
    SafetyFilter(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Terminal display error: {0}")]
    DisplayError(String),

    #[error("Invalid response format: {message}")]
    ResponseFormatError { message: String },

    #[allow(dead_code)]
    #[error("Request timeout")]
    Timeout,
}

impl ImagoError {
    /// Check if error is retryable (network/server errors)
    #[allow(dead_code)]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ImagoError::NetworkError(_)
                | ImagoError::ApiError {
                    status: 500..=599,
                    ..
                }
        )
    }
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, ImagoError>;
