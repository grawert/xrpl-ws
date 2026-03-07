use std::time::Duration;

use crate::connection::{ConnectionPools, LoadBalancing};

/// Configuration for XRPL WebSocket client behavior.
///
/// # Examples
///
/// Create a custom client config:
/// ```rust
/// use xrpl_ws::{XrplClient, ClientConfig};
/// use xrpl_ws::{ClientConfig, LoadBalancing};
///
/// let config = ClientConfig::default()
///     .with_request_timeout_secs(60)
///     .with_load_balancing(LoadBalancing::RoundRobin);
/// let client = XrplClient::with_config("wss://xrplcluster.com", config).await?;
/// ```
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout (default: 30 seconds)
    pub request_timeout: Duration,
    /// Size of the events broadcast channel buffer (default: 128)
    pub events_channel_size: usize,
    /// Size of the typed receiver broadcast channel buffer (default: 32)
    pub typed_receiver_channel_size: usize,
    /// Size of the command channel buffer (default: 32)
    pub cmd_channel_size: usize,
    /// Keepalive ping interval (default: 20 seconds)
    pub keepalive_interval: Duration,
    /// Initial backoff duration for reconnection attempts (default: 1 second)
    pub initial_backoff: Duration,
    /// Maximum backoff duration for reconnection attempts (default: 30 seconds)
    pub max_backoff: Duration,
    /// Connection pool configuration (default: smart defaults for zero-config)
    pub connection_pools: ConnectionPools,
    /// Load balancing strategy (default: RoundRobin)
    pub load_balancing: LoadBalancing,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            events_channel_size: 128,
            typed_receiver_channel_size: 32,
            cmd_channel_size: 32,
            keepalive_interval: Duration::from_secs(20),
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            connection_pools: ConnectionPools::default(),
            load_balancing: LoadBalancing::default(),
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

    /// Set the events broadcast channel size
    pub fn with_events_channel_size(mut self, size: usize) -> Self {
        self.events_channel_size = size;
        self
    }

    /// Set the typed receiver broadcast channel size
    pub fn with_typed_receiver_channel_size(mut self, size: usize) -> Self {
        self.typed_receiver_channel_size = size;
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

    /// Set the connection pool configuration
    pub fn with_connection_pools(mut self, pools: ConnectionPools) -> Self {
        self.connection_pools = pools;
        self
    }

    /// Set individual connection pool sizes (convenience method)
    pub fn with_pools(
        mut self,
        priority: usize,
        trading: usize,
        bulk: usize,
    ) -> Self {
        self.connection_pools = ConnectionPools { priority, trading, bulk };
        self
    }

    /// Set the load balancing strategy
    pub fn with_load_balancing(mut self, strategy: LoadBalancing) -> Self {
        self.load_balancing = strategy;
        self
    }

    /// Enable single connection mode (disables connection pooling)
    pub fn single_connection(mut self) -> Self {
        self.connection_pools =
            ConnectionPools { priority: 1, trading: 1, bulk: 1 };
        self
    }

    /// Enable high throughput mode (more connections for better performance)
    pub fn high_throughput(mut self) -> Self {
        self.connection_pools =
            ConnectionPools { priority: 2, trading: 4, bulk: 2 };
        self
    }
}
