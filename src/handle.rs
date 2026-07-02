use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::config::ClientConfig;
use crate::error::XrplError;
use crate::request::XrplSubscription;
use crate::socket::SubscribeRequest;

/// Unified event enum covering every subscription stream type this library
/// implements, dispatched on the wire `"type"` field. Lets callers using a
/// single [`SubscriptionHandle`] handle all events in one sequential match loop.
///
/// Unrecognized or unmodeled message types never surface here — they are
/// skipped transparently by [`SubscriptionStream::recv`].
#[derive(Debug, Clone, serde::Deserialize)]
#[non_exhaustive]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum SubscriptionEvent {
    /// A `ledgerClosed` message from the `ledger` stream.
    #[serde(rename = "ledgerClosed")]
    Ledger(crate::subscriptions::LedgerMessage),
    /// Covers the `accounts`/`accounts_proposed`, global transaction, and
    /// order-book streams — all three are tagged `"type": "transaction"`
    /// and share this message shape.
    #[serde(rename = "transaction")]
    Transaction(crate::subscriptions::AccountTransactionMessage),
    /// A `bookChanges` message from the `book_changes` stream.
    #[serde(rename = "bookChanges")]
    BookChanges(crate::subscriptions::BookChangesMessage),
}

/// Lets a [`SubscriptionHandle`] remotely close a [`SubscriptionStream`]:
/// [`SubscriptionHandle::close`] sets the flag, and the stream's `recv()`
/// checks it on every call to know when to stop yielding messages.
pub(crate) type ClosedFlag = Arc<AtomicBool>;

/// A single, type-scoped receiver over a shared subscription connection.
///
/// Holds the receive loop and liveness state so this logic exists in exactly
/// one place, whether the stream is scoped to one message type (e.g.
/// `SubscriptionStream<LedgerMessage>`) or unified (`SubscriptionStream<SubscriptionEvent>`).
pub struct SubscriptionStream<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    pub(crate) receiver: broadcast::Receiver<serde_json::Value>,
    pub(crate) closed: ClosedFlag,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> SubscriptionStream<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    /// Receive the next message from this stream.
    ///
    /// Deserialization failures are logged via `eprintln!` and skipped
    /// transparently — the library is expected to stay in sync with the
    /// XRPL protocol, so callers never need to handle schema-drift errors here.
    /// Returns [`XrplError::MessageDropped`] when this stream fell behind and
    /// messages were dropped, or [`XrplError::Disconnected`] when the
    /// connection is permanently closed.
    pub async fn recv(&mut self) -> Result<T, XrplError> {
        loop {
            if self.closed.load(Ordering::Relaxed) {
                return Err(XrplError::Disconnected);
            }

            match self.receiver.recv().await {
                Ok(value) => match serde_json::from_value::<T>(value.clone()) {
                    Ok(msg) => return Ok(msg),
                    Err(e) => {
                        eprintln!(
                            "Failed to deserialize subscription message: {e} - Raw: {value}"
                        );
                        continue;
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    return Err(XrplError::MessageDropped(n));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(XrplError::Disconnected);
                }
            }
        }
    }

    /// Manually close this stream. Also happens automatically when the
    /// owning [`SubscriptionHandle`] is closed or dropped.
    pub fn close(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

/// Handle for a shared connection over which one or more subscription
/// streams can be opened via [`subscribe`](Self::subscribe).
///
/// Dropping the handle closes the shared WebSocket connection — keep it
/// alive for as long as any derived [`SubscriptionStream`] is in use.
pub struct SubscriptionHandle<T = SubscriptionEvent>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    pub(crate) _connection: mpsc::Sender<SubscribeRequest>,
    pub(crate) config: ClientConfig,
    pub(crate) stream: SubscriptionStream<T>,
    pub(crate) closed_flags: Vec<ClosedFlag>,
}

impl<T> SubscriptionHandle<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    /// Receive the next message from this handle's stream.
    pub async fn recv(&mut self) -> Result<T, XrplError> {
        self.stream.recv().await
    }

    /// Manually close this handle and every [`SubscriptionStream`] derived from it.
    /// This is also called automatically when the handle is dropped.
    pub fn close(&mut self) {
        self.stream.close();
        for flag in &self.closed_flags {
            flag.store(true, Ordering::Relaxed);
        }
    }

    /// Open an additional subscription stream over this handle's shared connection.
    ///
    /// Returns the typed subscribe response and a [`SubscriptionStream`] scoped
    /// to this subscription's wire message type — isolated from any other
    /// subscription's traffic sharing the same connection.
    ///
    /// The returned stream's receiver is what keeps the subscription alive
    /// for replay on reconnect — dropping it (and every other receiver of
    /// this subscription's dedicated channel) lets the driver prune the
    /// subscription, so it will not be replayed after the next reconnect.
    /// Call `subscribe()` again for a second, independent consumer of the
    /// same stream.
    pub async fn subscribe<U>(
        &mut self,
        sub: &U,
    ) -> Result<(U::Response, SubscriptionStream<U::Message>), XrplError>
    where
        U: XrplSubscription,
        U::Message: Clone + Send + DeserializeOwned + Debug + 'static,
    {
        let (responder, rx) = oneshot::channel();
        let request = SubscribeRequest {
            request: sub.to_value(),
            responder,
            message_type: U::MESSAGE_TYPE,
        };

        self._connection
            .send(request)
            .await
            .map_err(|_| XrplError::Disconnected)?;

        let ack = tokio::time::timeout(self.config.request_timeout, rx)
            .await
            .map_err(|_| {
                XrplError::Timeout(
                    self.config.request_timeout.as_millis() as u64
                )
            })?
            .map_err(|_| XrplError::Disconnected)??;

        if let Some(err) = crate::api_error(&ack.response) {
            return Err(err);
        }

        let response: U::Response = serde_json::from_value(ack.response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        // Each stream gets its own closed flag so closing one doesn't close its siblings.
        let closed = ClosedFlag::new(AtomicBool::new(false));
        self.closed_flags.push(closed.clone());

        let stream = SubscriptionStream {
            receiver: ack.receiver,
            closed,
            _phantom: PhantomData,
        };

        Ok((response, stream))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn stream<T>(
        receiver: broadcast::Receiver<serde_json::Value>,
    ) -> SubscriptionStream<T>
    where
        T: Clone + Send + DeserializeOwned + Debug + 'static,
    {
        SubscriptionStream {
            receiver,
            closed: ClosedFlag::new(AtomicBool::new(false)),
            _phantom: PhantomData,
        }
    }

    /// Messages that fail to deserialize into `T` are skipped transparently —
    /// `recv()` never surfaces the schema mismatch, it just moves on to the
    /// next message on the stream.
    #[tokio::test]
    async fn recv_skips_undeserializable_messages_and_returns_next_valid_one() {
        let (tx, rx) = broadcast::channel(8);
        let mut stream = stream::<crate::subscriptions::LedgerMessage>(rx);

        let _ = tx.send(json!({ "type": "bookChanges", "unexpected": true }));
        let _ = tx.send(json!({
            "fee_base": 10,
            "ledger_hash": "ABC",
            "ledger_index": 5,
            "ledger_time": 1,
            "reserve_base": 1,
            "reserve_inc": 1,
            "txn_count": 0,
        }));

        let msg = stream
            .recv()
            .await
            .expect("should skip the bad message and return the valid one");
        assert_eq!(msg.ledger_index, 5);
    }

    #[tokio::test]
    async fn recv_returns_disconnected_once_closed() {
        let (_tx, rx) = broadcast::channel(8);
        let mut stream = stream::<crate::subscriptions::LedgerMessage>(rx);
        stream.close();

        let err = stream.recv().await.expect_err("closed stream must error");
        assert!(matches!(err, XrplError::Disconnected));
    }

    /// Regression test: derived streams must not share a single `closed`
    /// flag. Closing one stream must leave its siblings untouched, while
    /// closing the owning handle must still cascade to every stream it
    /// tracks in `closed_flags`.
    #[tokio::test]
    async fn closing_one_stream_does_not_close_its_siblings() {
        let (tx, _rx) = broadcast::channel::<serde_json::Value>(8);

        let mut stream_a = stream::<SubscriptionEvent>(tx.subscribe());
        let stream_b = stream::<SubscriptionEvent>(tx.subscribe());

        stream_a.close();

        assert!(stream_a.closed.load(Ordering::Relaxed));
        assert!(
            !stream_b.closed.load(Ordering::Relaxed),
            "closing one stream must not close an unrelated sibling stream"
        );

        let (conn_tx, _conn_rx) = mpsc::channel::<SubscribeRequest>(1);
        let mut handle = SubscriptionHandle {
            _connection: conn_tx,
            config: ClientConfig::default(),
            stream: stream::<SubscriptionEvent>(tx.subscribe()),
            closed_flags: vec![stream_b.closed.clone()],
        };

        handle.close();

        assert!(handle.stream.closed.load(Ordering::Relaxed));
        assert!(
            stream_b.closed.load(Ordering::Relaxed),
            "closing the handle must still cascade to every stream it tracks"
        );
    }
}
