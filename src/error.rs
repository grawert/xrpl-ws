use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum XrplError {
    #[error("Failed to connect: {0}")]
    ConnectionError(String),
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

impl From<tokio_tungstenite::tungstenite::Error> for XrplError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        XrplError::ConnectionError(e.to_string())
    }
}
