use thiserror::Error;

/// Errors that can be returned by the XRPL WebSocket client.
///
/// # Examples
///
/// ```no_run
/// use xrpl::{Client, XrplError};
/// use xrpl::request::account_info::AccountInfoRequest;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new("wss://xrplcluster.com");
///     let req = AccountInfoRequest {
///         account: "rBadAccount".to_string(),
///         ..Default::default()
///     };
///     match client.request(req).await {
///         Err(XrplError::ApiError { error, error_code, .. }) => {
///             eprintln!("rippled error {error} (code {error_code:?})");
///         }
///         Err(XrplError::Timeout(ms)) => eprintln!("timed out after {ms}ms"),
///         _ => {}
///     }
/// }
/// ```
#[derive(Error, Debug, Clone)]
pub enum XrplError {
    /// The WebSocket connection to the node could not be established.
    #[error("Failed to connect: {0}")]
    ConnectionError(String),
    /// The WebSocket connection was closed before the operation completed.
    #[error("WebSocket disconnected")]
    Disconnected,
    /// No response was received within the configured timeout period (milliseconds).
    #[error("Request timed out after {0}ms")]
    Timeout(u64),
    /// The server response could not be deserialized into the expected type.
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    /// The subscription channel fell behind and messages were dropped.
    /// The subscription is still active — call [`crate::SubscriptionHandle::recv`] again to continue.
    #[error("Subscription lagged: {0} messages dropped")]
    MessageDropped(u64),
    /// The rippled node returned an application-level error.
    #[error("API error '{error}': {}", error_message.as_deref().unwrap_or("no details"))]
    ApiError {
        /// Short error name returned by rippled (e.g. `"actNotFound"`).
        error: String,
        /// Numeric rippled error code (e.g. `23` for `actNotFound`), when present.
        error_code: Option<i32>,
        /// Human-readable description of the error, when present. Populated from
        /// `error_message` in the response, falling back to `error_exception` for
        /// internal rippled errors (neither field is in the official spec, but both
        /// are sent in practice).
        error_message: Option<String>,
    },
}

impl From<tokio_tungstenite::tungstenite::Error> for XrplError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        XrplError::ConnectionError(e.to_string())
    }
}
