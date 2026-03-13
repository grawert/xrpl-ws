use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{broadcast, mpsc};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::socket::SocketCommand;

/// Handle for managing a dedicated connection and its subscription.
/// Each handle owns its connection for complete isolation and automatic cleanup.
pub struct ConnectionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    _connection: mpsc::Sender<SocketCommand>,
    receiver: broadcast::Receiver<T>,
    closed: AtomicBool,
}

impl<T> ConnectionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    pub(crate) fn new(
        connection: mpsc::Sender<SocketCommand>,
        receiver: broadcast::Receiver<T>,
    ) -> Self {
        Self {
            _connection: connection,
            receiver,
            closed: AtomicBool::new(false),
        }
    }

    /// Receive the next message from this subscription.
    /// Returns an error if the connection is closed or on receive timeout/lag.
    pub async fn recv(&mut self) -> Result<T, broadcast::error::RecvError> {
        if self.closed.load(Ordering::Relaxed) {
            return Err(broadcast::error::RecvError::Closed);
        }

        self.receiver.recv().await
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

impl<T> Drop for ConnectionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    fn drop(&mut self) {
        self.close();
        // Connection mpsc::Sender is dropped, which closes the connection
    }
}
