use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};

use crate::config::ClientConfig;
use crate::socket::{ConnectionActor, SocketCommand};
use crate::error::XrplError;

/// Connection pool configuration for different XRPL data streams.
///
/// # Examples
///
/// Create a custom connection pool:
/// ```rust
/// use xrpl_ws::ConnectionPools;
/// let pools = ConnectionPools { priority: 2, trading: 4, bulk: 2 };
/// ```
///
/// Use with client config:
/// ```rust
/// use xrpl_ws::{ClientConfig, ConnectionPools};
/// let pools = ConnectionPools { priority: 2, trading: 4, bulk: 2 };
/// let config = ClientConfig::default().with_connection_pools(pools);
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionPools {
    /// Number of connections for time-critical subscriptions (default: 1)
    /// Used for ledger updates, price data that needs low latency
    pub priority: usize,
    /// Number of connections for trading data (default: 2)
    /// Used for order books, account updates that need moderate throughput
    pub trading: usize,
    /// Number of connections for bulk data streams (default: 1)
    /// Used for transaction streams, history data that needs high throughput
    pub bulk: usize,
}

impl Default for ConnectionPools {
    fn default() -> Self {
        Self {
            priority: 1, // Single fast connection for time-critical data
            trading: 2,  // Multiple connections for order book data
            bulk: 1,     // Single connection for bulk transaction streams
        }
    }
}

/// Load balancing strategy for connection selection within a pool
#[derive(Debug, Clone)]
pub enum LoadBalancing {
    /// Distribute subscriptions evenly across available connections
    RoundRobin,
    /// Route to connection with least active subscriptions
    LeastLoaded,
    /// Sticky routing - same subscription type always uses same connection
    Sticky,
}

impl Default for LoadBalancing {
    fn default() -> Self {
        LoadBalancing::RoundRobin
    }
}

/// Classification of subscription types for connection routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionClass {
    /// Time-critical subscriptions that need low latency
    Priority,
    /// Trading-related subscriptions that need moderate throughput
    Trading,
    /// High-volume subscriptions that need bulk throughput
    Bulk,
}

impl Default for SubscriptionClass {
    fn default() -> Self {
        SubscriptionClass::Trading
    }
}

/// Manages multiple WebSocket connections organized into pools
pub struct ConnectionManager {
    pools: HashMap<SubscriptionClass, Vec<mpsc::Sender<SocketCommand>>>,
    config: ClientConfig,
    load_balancer_state: HashMap<SubscriptionClass, usize>,
}

impl ConnectionManager {
    /// Create a new connection manager with the specified configuration
    pub async fn new(
        url: String,
        config: ClientConfig,
        subscriptions: Arc<DashMap<u64, Value>>,
    ) -> Result<(Self, broadcast::Sender<Value>), XrplError> {
        let (events_tx, _) = broadcast::channel(config.events_channel_size);
        let events_tx_clone = events_tx.clone();

        let mut pools = HashMap::new();
        let mut load_balancer_state = HashMap::new();

        // Initialize pools for each subscription class
        for &class in &[
            SubscriptionClass::Priority,
            SubscriptionClass::Trading,
            SubscriptionClass::Bulk,
        ] {
            let pool_size = match class {
                SubscriptionClass::Priority => config.connection_pools.priority,
                SubscriptionClass::Trading => config.connection_pools.trading,
                SubscriptionClass::Bulk => config.connection_pools.bulk,
            };

            let mut pool = Vec::new();

            // Create connections for this pool
            for _ in 0..pool_size {
                let (cmd_tx, pool_events_tx) = ConnectionActor::spawn(
                    url.clone(),
                    subscriptions.clone(),
                    config.clone(),
                );

                // Forward events from this connection to the main events channel
                Self::forward_events(
                    pool_events_tx.subscribe(),
                    events_tx_clone.clone(),
                );

                pool.push(cmd_tx);
            }

            pools.insert(class, pool);
            load_balancer_state.insert(class, 0);
        }

        Ok((Self { pools, config, load_balancer_state }, events_tx_clone))
    }

    /// Forward events from a pool connection to the main events channel
    fn forward_events(
        mut pool_events_rx: broadcast::Receiver<Value>,
        main_events_tx: broadcast::Sender<Value>,
    ) {
        tokio::spawn(async move {
            loop {
                match pool_events_rx.recv().await {
                    Ok(event) => {
                        let _ = main_events_tx.send(event);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Continue receiving, we can tolerate some lag in forwarding
                        continue;
                    }
                }
            }
        });
    }

    /// Get the appropriate connection for a subscription based on its class and load balancing
    pub fn get_connection(
        &mut self,
        class: SubscriptionClass,
    ) -> Option<&mpsc::Sender<SocketCommand>> {
        let pool = self.pools.get(&class)?;

        if pool.is_empty() {
            return None;
        }

        let index = match self.config.load_balancing {
            LoadBalancing::RoundRobin => {
                let current = self.load_balancer_state.get_mut(&class)?;
                let index = *current;
                *current = (*current + 1) % pool.len();
                index
            }
            LoadBalancing::LeastLoaded => {
                // For now, use round-robin. In a full implementation we'd track
                // active subscriptions per connection
                let current = self.load_balancer_state.get_mut(&class)?;
                let index = *current;
                *current = (*current + 1) % pool.len();
                index
            }
            LoadBalancing::Sticky => {
                // Always use the first connection for sticky routing
                0
            }
        };

        pool.get(index)
    }

    /// Send a command to the appropriate connection pool
    pub async fn send_command(
        &mut self,
        command: SocketCommand,
        class: SubscriptionClass,
    ) -> Result<(), XrplError> {
        if let Some(connection) = self.get_connection(class) {
            connection
                .send(command)
                .await
                .map_err(|_| XrplError::Disconnected)?;
            Ok(())
        } else {
            Err(XrplError::ConnectionError(
                "No available connections in pool".to_string(),
            ))
        }
    }
}
