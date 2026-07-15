use std::fmt::Debug;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::config::ClientConfig;
use crate::error::XrplError;
use crate::request::XrplSubscription;
use crate::socket::{
    SubscribeRequest, SubscriptionSessionRequest, UnsubscribeRequest,
};

/// Unified event over every subscription stream type, dispatched on the wire
/// `"type"` field. Lets a single [`SubscriptionSession`] handle all events in
/// one match loop.
///
/// Unrecognized `"type"` values deserialize into [`Unknown`](Self::Unknown).
/// A recognized `"type"` with a body that doesn't match its variant's shape
/// fails deserialization instead of falling back to `Unknown`.
#[derive(Debug, Clone)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum SubscriptionEvent {
    Ledger(crate::subscriptions::LedgerMessage),
    BookChanges(crate::subscriptions::BookChangesMessage),
    Transaction(crate::subscriptions::AccountTransactionMessage),
    Unknown { message_type: String, value: serde_json::Value },
}

impl<'de> serde::Deserialize<'de> for SubscriptionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;
        let message_type =
            value.get("type").and_then(serde_json::Value::as_str).unwrap_or("");

        match message_type {
            "ledgerClosed" => serde_json::from_value(value)
                .map(SubscriptionEvent::Ledger)
                .map_err(D::Error::custom),
            "transaction" => serde_json::from_value(value)
                .map(SubscriptionEvent::Transaction)
                .map_err(D::Error::custom),
            "bookChanges" => serde_json::from_value(value)
                .map(SubscriptionEvent::BookChanges)
                .map_err(D::Error::custom),
            other => Ok(SubscriptionEvent::Unknown {
                message_type: other.to_string(),
                value,
            }),
        }
    }
}

/// A type-scoped receiver over a shared subscription connection, scoped to
/// one subscription's message type (`SubscriptionStream<LedgerMessage>`) or
/// unified over all of a session's subscriptions (`SubscriptionStream<SubscriptionEvent>`).
///
/// Independently owned via its own `connection` sender clone - keeps working
/// after the [`SubscriptionSession`] that created it is dropped. Drop it to
/// stop locally, or call [`unsubscribe`](Self::unsubscribe) to also stop the
/// server side.
pub struct SubscriptionStream<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    /// Request id from this stream's `subscribe` call; `None` for the
    /// session's built-in unified stream.
    pub(crate) id: Option<u64>,
    pub(crate) receiver: broadcast::Receiver<serde_json::Value>,
    /// Sends this stream's `unsubscribe` request; also keeps the connection
    /// driver alive while held.
    pub(crate) connection: mpsc::Sender<SubscriptionSessionRequest>,
    pub(crate) _phantom: PhantomData<T>,
}

impl<T> SubscriptionStream<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    /// Receive the next message from this stream.
    ///
    /// Deserialization failures are logged via `eprintln!` and skipped.
    /// Returns [`XrplError::MessageDropped`] if this stream fell behind, or
    /// [`XrplError::Disconnected`] once the connection is closed.
    pub async fn recv(&mut self) -> Result<T, XrplError> {
        loop {
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

    /// Tells the server to stop this subscription and awaits its
    /// acknowledgement. The local registration is removed immediately, so it
    /// won't be replayed on reconnect. For a fire-and-forget stop, drop the
    /// stream instead.
    ///
    /// rippled subscriptions are per-connection, not per-caller: this also
    /// silences any other stream independently subscribed to the same topic
    /// on the same connection.
    pub async fn unsubscribe(mut self) -> Result<(), XrplError> {
        let Some(id) = self.id.take() else {
            return Ok(()); // nothing was ever subscribed on this stream
        };

        let (responder, rx) = oneshot::channel();
        self.connection
            .send(SubscriptionSessionRequest::Unsubscribe(UnsubscribeRequest {
                id,
                responder,
            }))
            .await
            .map_err(|_| XrplError::Disconnected)?;

        let value = rx.await.map_err(|_| XrplError::Disconnected)??;
        if let Some(err) = crate::rippled_error(&value) {
            return Err(err);
        }
        Ok(())
    }
}

impl<T> Drop for SubscriptionStream<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    fn drop(&mut self) {
        // Mirrors `unsubscribe()` but fire-and-forget; no-op if `unsubscribe()` already ran.
        if let Some(id) = self.id {
            let (responder, _rx) = oneshot::channel();
            let _ = self.connection.try_send(
                SubscriptionSessionRequest::Unsubscribe(UnsubscribeRequest {
                    id,
                    responder,
                }),
            );
        }
    }
}

/// Session over a shared connection on which subscription streams can be
/// opened via [`subscribe`](Self::subscribe).
///
/// Each derived [`SubscriptionStream`] is independently owned; dropping this
/// session only gives up the ability to open further subscriptions, it does
/// not affect streams already handed out. Manage each stream's own lifecycle
/// via [`unsubscribe`](SubscriptionStream::unsubscribe) or by dropping it.
pub struct SubscriptionSession<T = SubscriptionEvent>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    pub(crate) config: ClientConfig,
    pub(crate) stream: SubscriptionStream<T>,
    pub(crate) _connection: mpsc::Sender<SubscriptionSessionRequest>,
}

impl<T> SubscriptionSession<T>
where
    T: Clone + Send + DeserializeOwned + Debug + 'static,
{
    /// Receive the next message from this session's stream.
    pub async fn recv(&mut self) -> Result<T, XrplError> {
        self.stream.recv().await
    }

    /// Open an additional subscription stream over this session's shared connection.
    ///
    /// Returns the typed subscribe response and a [`SubscriptionStream`]
    /// scoped to this subscription's message type, isolated from other
    /// subscriptions on the same connection. The stream outlives this
    /// session. Dropping it (or calling [`SubscriptionStream::unsubscribe`])
    /// stops the subscription server-side too.
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
            request: sub.to_value()?,
            responder,
            message_type: U::MESSAGE_TYPE,
        };

        self._connection
            .send(SubscriptionSessionRequest::Subscribe(request))
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

        if let Some(err) = crate::rippled_error(&ack.response) {
            return Err(err);
        }

        let response: U::Response = serde_json::from_value(ack.response)
            .map_err(|e| XrplError::ParseError(e.to_string()))?;

        let stream = SubscriptionStream {
            id: Some(ack.id),
            receiver: ack.receiver,
            connection: self._connection.clone(),
            _phantom: PhantomData,
        };

        Ok((response, stream))
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
        let (connection, _req_rx) = mpsc::channel(1);
        SubscriptionStream {
            id: None,
            receiver,
            connection,
            _phantom: PhantomData,
        }
    }

    /// A push with a `"type"` this library doesn't model deserializes into
    /// `Unknown`, preserving the type string and the full raw message.
    #[test]
    fn unrecognized_type_deserializes_into_unknown() {
        let value = json!({ "type": "peerStatusChange", "foo": "bar" });
        let event: SubscriptionEvent =
            serde_json::from_value(value.clone()).expect("must not error");

        match event {
            SubscriptionEvent::Unknown { message_type, value: raw } => {
                assert_eq!(message_type, "peerStatusChange");
                assert_eq!(raw, value);
            }
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    /// A recognized `"type"` whose body doesn't match that variant's shape
    /// is a schema mismatch, not an unmodelled stream, so it surfaces as a
    /// deserialization error rather than `Unknown`.
    #[test]
    fn recognized_type_with_malformed_body_still_errors() {
        let value =
            json!({ "type": "ledgerClosed", "ledger_index": "not a number" });
        let result: Result<SubscriptionEvent, _> =
            serde_json::from_value(value);

        assert!(
            result.is_err(),
            "a known type with a malformed body must error, not fall back to Unknown"
        );
    }

    /// Messages that fail to deserialize into `T` are skipped transparently -
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

    /// Once every sender for the underlying broadcast channel is dropped,
    /// `recv()` surfaces that as `Disconnected`.
    #[tokio::test]
    async fn recv_returns_disconnected_once_channel_closes() {
        let (tx, rx) = broadcast::channel(8);
        let mut stream = stream::<crate::subscriptions::LedgerMessage>(rx);
        drop(tx);

        let err = stream.recv().await.expect_err("closed channel must error");
        assert!(matches!(err, XrplError::Disconnected));
    }

    /// A stream returned by `subscribe()` is independently owned and holds
    /// its own connection reference, so it must keep receiving messages
    /// even after the session that created it is dropped.
    #[tokio::test]
    async fn stream_outlives_the_session_that_created_it() {
        let (tx, _rx) = broadcast::channel::<serde_json::Value>(8);
        let (conn_tx, _conn_rx) =
            mpsc::channel::<SubscriptionSessionRequest>(1);

        let session = SubscriptionSession {
            config: ClientConfig::default(),
            stream: stream::<SubscriptionEvent>(tx.subscribe()),
            _connection: conn_tx,
        };
        let mut derived = stream::<SubscriptionEvent>(tx.subscribe());

        drop(session);

        let _ = tx.send(json!({
            "type": "ledgerClosed",
            "fee_base": 10,
            "ledger_hash": "ABC",
            "ledger_index": 1,
            "ledger_time": 1,
            "reserve_base": 1,
            "reserve_inc": 1,
            "txn_count": 0,
        }));
        derived.recv().await.expect(
            "stream must still receive messages after the session is dropped",
        );
    }

    /// Explicit `unsubscribe()` sends the request over the stream's own
    /// connection sender, closes the stream locally, and waits for the
    /// server's acknowledgement before resolving.
    #[tokio::test]
    async fn unsubscribe_sends_request_and_awaits_ack() {
        let (_tx, rx) = broadcast::channel(8);
        let (connection, mut req_rx) = mpsc::channel(1);
        let stream = SubscriptionStream::<SubscriptionEvent> {
            id: Some(42),
            receiver: rx,
            connection,
            _phantom: PhantomData,
        };

        let responder = tokio::spawn(async move {
            let SubscriptionSessionRequest::Unsubscribe(UnsubscribeRequest {
                id,
                responder,
            }) = req_rx
                .recv()
                .await
                .expect("must receive the unsubscribe request")
            else {
                panic!("expected an Unsubscribe request");
            };
            assert_eq!(id, 42);
            let _ = responder.send(Ok(json!({ "status": "success" })));
        });

        stream.unsubscribe().await.expect("unsubscribe must succeed");
        responder.await.unwrap();
    }

    /// Dropping a stream without calling `unsubscribe()` still notifies the
    /// server, best-effort.
    #[test]
    fn drop_fires_best_effort_unsubscribe() {
        let (_tx, rx) = broadcast::channel(8);
        let (connection, mut req_rx) = mpsc::channel(1);
        let stream = SubscriptionStream::<SubscriptionEvent> {
            id: Some(7),
            receiver: rx,
            connection,
            _phantom: PhantomData,
        };

        drop(stream);

        let SubscriptionSessionRequest::Unsubscribe(UnsubscribeRequest {
            id,
            ..
        }) = req_rx.try_recv().expect("drop must send an unsubscribe request")
        else {
            panic!("expected an Unsubscribe request");
        };
        assert_eq!(id, 7);
    }

    /// The session's built-in unified stream was never itself subscribed to
    /// anything (`id: None`), so `unsubscribe()` on it is a harmless no-op -
    /// nothing is sent over the connection.
    #[tokio::test]
    async fn unsubscribe_with_no_id_is_a_noop() {
        let (_tx, rx) = broadcast::channel(8);
        let (connection, mut req_rx) = mpsc::channel(1);
        let stream = SubscriptionStream::<SubscriptionEvent> {
            id: None,
            receiver: rx,
            connection,
            _phantom: PhantomData,
        };

        stream.unsubscribe().await.expect("must resolve Ok with nothing to do");
        assert!(
            req_rx.try_recv().is_err(),
            "nothing should be sent when there was never a subscription"
        );
    }
}
