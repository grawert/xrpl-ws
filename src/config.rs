use std::time::Duration;

/// Configuration for XRPL WebSocket client behavior.
///
/// # Example
///
/// Create a custom client config:
/// ```rust
/// use xrpl::{Client, ClientConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), xrpl::XrplError> {
///     let config = ClientConfig::default()
///         .with_request_timeout_secs(60)
///         .with_keepalive_secs(30);
///     let client = Client::with_config("wss://xrplcluster.com", config);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Size of the command channel buffer (default: 32)
    pub cmd_channel_size: usize,
    /// Size of the subscription channel buffer (default: 32)
    pub subscription_channel_size: usize,
    /// Request timeout (default: 30 seconds)
    pub request_timeout: Duration,
    /// Keepalive ping interval (default: 20 seconds)
    pub keepalive_interval: Duration,
    /// Initial backoff duration for reconnection attempts (default: 1 second)
    pub initial_backoff: Duration,
    /// Maximum backoff duration for reconnection attempts (default: 30 seconds)
    pub max_backoff: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            subscription_channel_size: 32,
            cmd_channel_size: 32,
            keepalive_interval: Duration::from_secs(20),
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
        }
    }
}

impl ClientConfig {
    /// Create a new ClientConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set the request timeout in seconds (convenience method)
    pub fn with_request_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.request_timeout = Duration::from_secs(timeout_secs);
        self
    }

    /// Set the subscription channel size
    pub fn with_subscription_channel_size(mut self, size: usize) -> Self {
        self.subscription_channel_size = size;
        self
    }

    /// Set the command channel size
    pub fn with_cmd_channel_size(mut self, size: usize) -> Self {
        self.cmd_channel_size = size;
        self
    }

    /// Set the keepalive interval
    pub fn with_keepalive_interval(mut self, interval: Duration) -> Self {
        self.keepalive_interval = interval;
        self
    }

    /// Set the keepalive interval in seconds (convenience method)
    pub fn with_keepalive_secs(mut self, interval_secs: u64) -> Self {
        self.keepalive_interval = Duration::from_secs(interval_secs);
        self
    }

    /// Set the initial backoff duration
    pub fn with_initial_backoff(mut self, backoff: Duration) -> Self {
        self.initial_backoff = backoff;
        self
    }

    /// Set the initial backoff duration in seconds (convenience method)
    pub fn with_initial_backoff_secs(mut self, backoff_secs: u64) -> Self {
        self.initial_backoff = Duration::from_secs(backoff_secs);
        self
    }

    /// Set the maximum backoff duration
    pub fn with_max_backoff(mut self, backoff: Duration) -> Self {
        self.max_backoff = backoff;
        self
    }

    /// Set the maximum backoff duration in seconds (convenience method)
    pub fn with_max_backoff_secs(mut self, backoff_secs: u64) -> Self {
        self.max_backoff = Duration::from_secs(backoff_secs);
        self
    }
}
