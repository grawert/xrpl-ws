use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{broadcast, mpsc};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::XrplError;
use crate::socket::SocketRequest;

/// Handle for managing a dedicated connection and its subscription.
/// Each handle owns its connection for complete isolation and automatic cleanup.
pub struct SubscriptionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    _connection: mpsc::Sender<SocketRequest>,
    receiver: broadcast::Receiver<T>,
    closed: AtomicBool,
}

impl<T> SubscriptionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    pub(crate) fn new(
        connection: mpsc::Sender<SocketRequest>,
        receiver: broadcast::Receiver<T>,
    ) -> Self {
        Self {
            _connection: connection,
            receiver,
            closed: AtomicBool::new(false),
        }
    }

    /// Receive the next message from this subscription.
    ///
    /// Returns [`XrplError::MessageDropped`] when the internal channel fell behind
    /// and messages were lost — the subscription is still active and calling `recv`
    /// again will continue from the next available message.
    /// Returns [`XrplError::Disconnected`] when the connection is permanently closed.
    pub async fn recv(&mut self) -> Result<T, XrplError> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(XrplError::Disconnected);
        }

        match self.receiver.recv().await {
            Ok(msg) => Ok(msg),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                Err(XrplError::MessageDropped(n))
            }
            Err(broadcast::error::RecvError::Closed) => {
                Err(XrplError::Disconnected)
            }
        }
    }

    /// Get a clone of the receiver for this subscription.
    /// Useful if you want multiple consumers of the same subscription stream.
    pub fn resubscribe(&self) -> broadcast::Receiver<T> {
        self.receiver.resubscribe()
    }

    /// Manually close this connection handle.
    /// This is also called automatically when the handle is dropped.
    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
        // Connection will be dropped, closing the WebSocket
    }
}

impl<T> Drop for SubscriptionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    fn drop(&mut self) {
        self.close();
        // Connection mpsc::Sender is dropped, which closes the connection
    }
}
