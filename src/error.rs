use thiserror::Error;

#[derive(Error, Debug)]
pub enum XrplError {
    #[error("Failed to connect: {0}")]
    ConnectionError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("WebSocket disconnected")]
    Disconnected,

    #[error("Request timed out after {0}ms")]
    Timeout(u64),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("API error '{error}': {}", error_message.as_deref().unwrap_or("no details"))]
    ApiError {
        error: String,
        error_code: Option<i32>,
        error_message: Option<String>,
    },
}
