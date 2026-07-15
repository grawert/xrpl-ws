use std::collections::HashMap;
use std::ops::ControlFlow;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Error as WsError},
    MaybeTlsStream, WebSocketStream,
};

use crate::config::ClientConfig;
use crate::error::XrplError;

#[cfg(feature = "jsondump")]
macro_rules! json_dump {
    ($label:expr, $value:expr) => {
        if let Ok(json) = serde_json::to_string_pretty($value) {
            use std::io::Write;
            let _ = writeln!(
                std::io::stderr().lock(),
                "\n=== {}\n{}\n===\n",
                $label,
                json
            );
        }
    };
}

#[cfg(not(feature = "jsondump"))]
macro_rules! json_dump {
    ($label:expr, $value:expr) => {};
}

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;

#[derive(Debug)]
pub(crate) struct SocketRequest {
    pub(crate) request: Value,
    pub(crate) responder: oneshot::Sender<Result<Value, XrplError>>,
}

/// Request to open a subscription over the shared connection. Carries the
/// wire message type up front so the driver can route incoming pushes to
/// this subscription's own dedicated broadcast channel.
pub(crate) struct SubscribeRequest {
    pub(crate) request: Value,
    pub(crate) responder: oneshot::Sender<Result<SubscribeAck, XrplError>>,
    pub(crate) message_type: &'static str,
}

/// A single active subscription: its replay payload, its wire message
/// type (for routing incoming pushes), and a dedicated broadcast sender
/// for its own stream. Each subscription gets its own channel - it is
/// no longer shared with other subscriptions of the same message_type.
/// Liveness (for both delivery pruning and reconnect-replay) is derived
/// directly from this channel via `receiver_count()` / failed `send()` -
/// no separate liveness token is tracked.
struct Subscription {
    sender: broadcast::Sender<Value>,
    payload: Value,
    message_type: &'static str,
}

/// Successful outcome of a [`SubscribeRequest`]: the raw subscribe response
/// and a receiver scoped to this subscription's own dedicated channel.
pub(crate) struct SubscribeAck {
    /// Request id assigned to the originating `subscribe` call, needed to
    /// later tell the driver which [`Subscription`] entry to unsubscribe.
    pub(crate) id: u64,
    pub(crate) response: Value,
    pub(crate) receiver: broadcast::Receiver<Value>,
}

/// Request to end a previously opened subscription, identified by the
/// request id its `subscribe` call was assigned. The driver rebuilds the
/// wire `unsubscribe` payload from the matching [`Subscription`]'s own
/// stored payload, so the caller never needs to resend `streams`/`accounts`/
/// `books` fields itself.
pub(crate) struct UnsubscribeRequest {
    pub(crate) id: u64,
    pub(crate) responder: oneshot::Sender<Result<Value, XrplError>>,
}

/// Every message a [`SubscriptionSession`] accepts over its request channel.
pub(crate) enum SubscriptionSessionRequest {
    Subscribe(SubscribeRequest),
    Unsubscribe(UnsubscribeRequest),
}

/// Responder for a pending subscription session request. Subscribe and
/// unsubscribe requests resolve to different payload types, so the pending
/// map wraps whichever oneshot sender the original request carried.
enum SessionResponder {
    Subscribe(oneshot::Sender<Result<SubscribeAck, XrplError>>),
    Unsubscribe(oneshot::Sender<Result<Value, XrplError>>),
}

impl SessionResponder {
    fn send_err(self, e: XrplError) {
        match self {
            Self::Subscribe(tx) => {
                let _ = tx.send(Err(e));
            }
            Self::Unsubscribe(tx) => {
                let _ = tx.send(Err(e));
            }
        }
    }
}

/// Tracks in-flight one-shot requests using a uniform result type.
struct PendingRequests {
    counter: u64,
    pending: HashMap<u64, oneshot::Sender<Result<Value, XrplError>>>,
}

impl PendingRequests {
    fn new() -> Self {
        Self { counter: 0, pending: HashMap::new() }
    }

    fn next_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    fn register(
        &mut self,
        mut request: Value,
        responder: oneshot::Sender<Result<Value, XrplError>>,
    ) -> Value {
        request["id"] = self.next_id().into();
        self.pending.insert(self.counter, responder);
        json_dump!("REQUEST", &request);
        request
    }

    fn resolve(
        &mut self,
        id: u64,
    ) -> Option<oneshot::Sender<Result<Value, XrplError>>> {
        self.pending.remove(&id)
    }

    fn cancel_all(&mut self) {
        for (_, responder) in self.pending.drain() {
            let _ = responder.send(Err(XrplError::Disconnected));
        }
    }
}

/// Tracks in-flight `subscribe`/`unsubscribe` requests for a subscription
/// handler using the heterogeneous [`SessionResponder`].
struct PendingSubscriptions {
    counter: u64,
    pending: HashMap<u64, SessionResponder>,
}

impl PendingSubscriptions {
    fn new() -> Self {
        Self { counter: 0, pending: HashMap::new() }
    }

    fn next_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    fn register(
        &mut self,
        mut request: Value,
        responder: SessionResponder,
    ) -> Value {
        request["id"] = self.next_id().into();
        self.pending.insert(self.counter, responder);
        json_dump!("REQUEST", &request);
        request
    }

    fn resolve(&mut self, id: u64) -> Option<SessionResponder> {
        self.pending.remove(&id)
    }

    fn cancel_all(&mut self) {
        for (_, responder) in self.pending.drain() {
            responder.send_err(XrplError::Disconnected);
        }
    }
}

/// Protocol delegate for a single WebSocket session.
///
/// [`ConnectionDriver`] calls these methods as connection events occur.
/// Implementors encode the protocol logic (request tracking, subscription
/// replay, disconnect cleanup) without owning the connection itself.
///
/// Methods that can end the session return `ControlFlow<String>` rather than
/// `ControlFlow<()>`: the `Break` payload is a ready-to-log reason, so the
/// caller that ultimately decides to stop (`ConnectionDriver::run_session`)
/// can just `eprintln!("{reason}")` instead of reconstructing why from a
/// bare unit value and a comment.
trait SessionHandler: Send + 'static {
    /// The type of requests submitted to this session over its mpsc channel.
    type Message: Send + 'static;

    fn on_connect(&mut self) -> Vec<Value>;
    fn on_request(&mut self, req: Self::Message) -> Option<Value>;
    fn on_response(&mut self, value: Value) -> ControlFlow<String>;
    fn on_disconnect(&mut self);
}

/// Drives a persistent, auto-reconnecting WebSocket connection.
///
/// Owns the connection lifecycle: connects, delegates session events to an
/// [`SessionHandler`], and reconnects with exponential backoff on failure.
struct ConnectionDriver<H: SessionHandler> {
    url: String,
    config: ClientConfig,
    req_rx: mpsc::Receiver<H::Message>,
    handler: H,
}

impl<H: SessionHandler> ConnectionDriver<H> {
    fn new(
        url: String,
        config: ClientConfig,
        req_rx: mpsc::Receiver<H::Message>,
        handler: H,
    ) -> Self {
        Self { url, config, req_rx, handler }
    }

    async fn run(mut self) {
        let mut backoff = self.config.initial_backoff;

        loop {
            match connect_async(&self.url).await {
                Ok((ws_stream, _)) => {
                    backoff = self.config.initial_backoff;
                    self.run_session(ws_stream).await;
                }
                Err(e) => {
                    eprintln!(
                        "WS Connect Error: {e}. Retrying in {}s",
                        backoff.as_secs()
                    );
                }
            }

            if let ControlFlow::Break(reason) =
                self.reconnect(&mut backoff).await
            {
                eprintln!("Connection driver stopping: {reason}");
                break;
            }
        }
    }

    async fn reconnect(
        &mut self,
        backoff: &mut Duration,
    ) -> ControlFlow<String> {
        self.handler.on_disconnect();

        if self.req_rx.is_closed() {
            return ControlFlow::Break(
                "request sender dropped - shutting down".to_string(),
            );
        }

        tokio::time::sleep(*backoff).await;
        *backoff = (*backoff * 2).min(self.config.max_backoff);
        ControlFlow::Continue(())
    }

    async fn run_session(&mut self, ws_stream: WsStream) {
        let (mut write, mut read) = ws_stream.split();
        let mut ping_interval =
            tokio::time::interval(self.config.keepalive_interval);

        for msg in self.handler.on_connect() {
            let _ = write.send(Message::Text(msg.to_string().into())).await;
        }

        loop {
            let alive = tokio::select! {
                _   = ping_interval.tick() => self.ping(&mut write).await,
                req = self.req_rx.recv()   => self.request(&mut write, req).await,
                msg = read.next()          => self.response(&mut write, msg).await,
            };
            if let ControlFlow::Break(reason) = alive {
                eprintln!("Session ending: {reason}");
                break;
            }
        }
    }

    async fn ping(&mut self, write: &mut WsSink) -> ControlFlow<String> {
        let empty = vec![].into();
        match write.send(Message::Ping(empty)).await {
            Ok(()) => ControlFlow::Continue(()),
            Err(e) => ControlFlow::Break(format!("failed to send ping: {e}")),
        }
    }

    async fn request(
        &mut self,
        write: &mut WsSink,
        req: Option<H::Message>,
    ) -> ControlFlow<String> {
        let Some(req) = req else {
            return ControlFlow::Break(
                "request sender dropped - shutting down".to_string(),
            );
        };
        let Some(request) = self.handler.on_request(req) else {
            return ControlFlow::Continue(()); // handler chose not to send
        };
        match write.send(Message::Text(request.to_string().into())).await {
            Ok(()) => ControlFlow::Continue(()),
            Err(e) => {
                ControlFlow::Break(format!("failed to send request: {e}"))
            }
        }
    }

    async fn response(
        &mut self,
        write: &mut WsSink,
        msg: Option<Result<Message, WsError>>,
    ) -> ControlFlow<String> {
        match msg {
            Some(Ok(Message::Text(json))) => {
                match serde_json::from_str::<Value>(&json) {
                    Ok(value) => return self.handler.on_response(value),
                    Err(e) => eprintln!(
                        "Failed to parse incoming JSON: {e} - Raw: {json}"
                    ),
                }
                ControlFlow::Continue(())
            }
            Some(Ok(Message::Ping(data))) => {
                let _ = write.send(Message::Pong(data)).await;
                ControlFlow::Continue(())
            }
            Some(Err(e)) => {
                ControlFlow::Break(format!("WebSocket read error: {e}"))
            }
            None => {
                ControlFlow::Break("WebSocket read stream closed".to_string())
            }
            _ => ControlFlow::Continue(()),
        }
    }
}

struct RequestHandler {
    pending: PendingRequests,
}

impl RequestHandler {
    fn new() -> Self {
        Self { pending: PendingRequests::new() }
    }
}

impl SessionHandler for RequestHandler {
    type Message = SocketRequest;

    fn on_connect(&mut self) -> Vec<Value> {
        Vec::new()
    }

    fn on_request(&mut self, req: SocketRequest) -> Option<Value> {
        let SocketRequest { request, responder } = req;
        Some(self.pending.register(request, responder))
    }

    fn on_response(&mut self, value: Value) -> ControlFlow<String> {
        if let Some(id) = value["id"].as_u64() {
            json_dump!("RESPONSE", &value);
            if let Some(responder) = self.pending.resolve(id) {
                let _ = responder.send(Ok(value));
                return ControlFlow::Continue(());
            }
            json_dump!("UNMATCHED_RESPONSE", &value);
            return ControlFlow::Break(format!(
                "protocol violation: unmatched response id {id}"
            ));
        }
        eprintln!("Unexpected message without id: {value}");
        ControlFlow::Continue(())
    }

    fn on_disconnect(&mut self) {
        self.pending.cancel_all();
    }
}

struct SubscriptionHandler {
    requests: PendingSubscriptions,
    stream_tx: broadcast::Sender<Value>,
    channel_size: usize,
    subscriptions: HashMap<u64, Subscription>,
    pending_receivers: HashMap<u64, broadcast::Receiver<Value>>,
}

impl SubscriptionHandler {
    fn new(stream_tx: broadcast::Sender<Value>, channel_size: usize) -> Self {
        Self {
            requests: PendingSubscriptions::new(),
            stream_tx,
            channel_size,
            subscriptions: HashMap::new(),
            pending_receivers: HashMap::new(),
        }
    }
}

impl SessionHandler for SubscriptionHandler {
    type Message = SubscriptionSessionRequest;

    fn on_connect(&mut self) -> Vec<Value> {
        self.subscriptions.retain(|_, sub| sub.sender.receiver_count() > 0);
        self.subscriptions.values().map(|sub| sub.payload.clone()).collect()
    }

    fn on_request(&mut self, req: SubscriptionSessionRequest) -> Option<Value> {
        match req {
            SubscriptionSessionRequest::Subscribe(SubscribeRequest {
                request,
                responder,
                message_type,
            }) => {
                let payload = self
                    .requests
                    .register(request, SessionResponder::Subscribe(responder));
                let id = payload["id"]
                    .as_u64()
                    .expect("PendingRequests::register always assigns an id");

                let (sender, receiver) = broadcast::channel(self.channel_size);
                self.subscriptions.insert(
                    id,
                    Subscription {
                        payload: payload.clone(),
                        message_type,
                        sender,
                    },
                );
                self.pending_receivers.insert(id, receiver);

                Some(payload)
            }
            SubscriptionSessionRequest::Unsubscribe(UnsubscribeRequest {
                id,
                responder,
            }) => {
                let Some(sub) = self.subscriptions.remove(&id) else {
                    let _ = responder.send(Ok(Value::Null));
                    return None;
                };

                let mut request = sub.payload;
                request["command"] = "unsubscribe".into();

                let payload = self.requests.register(
                    request,
                    SessionResponder::Unsubscribe(responder),
                );
                Some(payload)
            }
        }
    }

    fn on_response(&mut self, value: Value) -> ControlFlow<String> {
        match value["id"].as_u64() {
            Some(id) => {
                json_dump!("RESPONSE", &value);
                if let Some(responder) = self.requests.resolve(id) {
                    match responder {
                        SessionResponder::Subscribe(tx) => {
                            if let Some(receiver) =
                                self.pending_receivers.remove(&id)
                            {
                                let _ = tx.send(Ok(SubscribeAck {
                                    response: value,
                                    receiver,
                                    id,
                                }));
                            }
                        }
                        SessionResponder::Unsubscribe(tx) => {
                            let _ = tx.send(Ok(value));
                        }
                    }
                    return ControlFlow::Continue(());
                }
                json_dump!("UNMATCHED_RESPONSE", &value);
                ControlFlow::Break(format!(
                    "protocol violation: unmatched response id {id}"
                ))
            }
            None => {
                json_dump!("PUSH_MESSAGE", &value);
                let _ = self.stream_tx.send(value.clone());

                let message_type = value.get("type").and_then(Value::as_str);
                self.subscriptions.retain(|_, sub| {
                    if Some(sub.message_type) != message_type {
                        true
                    } else {
                        sub.sender.send(value.clone()).is_ok()
                    }
                });

                ControlFlow::Continue(())
            }
        }
    }

    fn on_disconnect(&mut self) {
        self.requests.cancel_all();
        self.pending_receivers.clear();
    }
}

/// Spawns a persistent, multiplexed WebSocket connection for one-shot requests.
/// Returns a sender that routes each [`SocketRequest`] through the shared connection.
pub(crate) fn request(
    url: String,
    config: ClientConfig,
) -> mpsc::Sender<SocketRequest> {
    let (req_tx, req_rx) = mpsc::channel(config.cmd_channel_size);
    let handler = RequestHandler::new();
    tokio::spawn(ConnectionDriver::new(url, config, req_rx, handler).run());
    req_tx
}

/// Spawns a persistent, multiplexed WebSocket connection shared by all
/// subscriptions issued over it. Returns a sender for issuing
/// [`SubscriptionSessionRequest`]s and the umbrella broadcast receiver carrying
/// every pushed message, untyped, regardless of wire message type.
pub(crate) fn subscribe(
    url: String,
    config: ClientConfig,
) -> (mpsc::Sender<SubscriptionSessionRequest>, broadcast::Receiver<Value>) {
    let (req_tx, req_rx) = mpsc::channel(config.cmd_channel_size);
    let (stream_tx, stream_rx) =
        broadcast::channel(config.subscription_channel_size);

    let handler =
        SubscriptionHandler::new(stream_tx, config.subscription_channel_size);

    tokio::spawn(ConnectionDriver::new(url, config, req_rx, handler).run());

    (req_tx, stream_rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn acknowledge(
        handler: &mut SubscriptionHandler,
        message_type: &'static str,
    ) -> SubscribeAck {
        let (responder, mut rx) = oneshot::channel();
        let payload = handler
            .on_request(SubscriptionSessionRequest::Subscribe(
                SubscribeRequest {
                    request: json!({ "command": "subscribe" }),
                    responder,
                    message_type,
                },
            ))
            .expect("on_request must produce a payload to send");
        let id = payload["id"].as_u64().expect("payload must carry an id");

        let _ = handler.on_response(json!({ "id": id, "status": "success" }));

        rx.try_recv()
            .expect("ack must be delivered synchronously")
            .expect("ack must not be an error")
    }

    /// Two subscriptions of different wire message types, registered over the
    /// same session, must each only ever observe their own type of push
    /// message - never the other's - while the umbrella channel sees both.
    #[test]
    fn concurrent_subscriptions_are_isolated_by_message_type() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);
        let mut umbrella = handler.stream_tx.subscribe();

        let ledger_ack = acknowledge(&mut handler, "ledgerClosed");
        let tx_ack = acknowledge(&mut handler, "transaction");
        let mut ledger_rx = ledger_ack.receiver;
        let mut tx_rx = tx_ack.receiver;

        let _ = handler.on_response(
            json!({ "type": "ledgerClosed", "ledger_index": 100 }),
        );
        let _ = handler
            .on_response(json!({ "type": "transaction", "hash": "ABC" }));

        let ledger_msg = ledger_rx
            .try_recv()
            .expect("ledger stream should receive its message");
        assert_eq!(ledger_msg["type"], "ledgerClosed");
        assert!(
            matches!(
                ledger_rx.try_recv(),
                Err(broadcast::error::TryRecvError::Empty)
            ),
            "ledger stream must not observe transaction traffic"
        );

        let tx_msg =
            tx_rx.try_recv().expect("tx stream should receive its message");
        assert_eq!(tx_msg["type"], "transaction");
        assert!(
            matches!(
                tx_rx.try_recv(),
                Err(broadcast::error::TryRecvError::Empty)
            ),
            "transaction stream must not observe ledger traffic"
        );

        // The umbrella channel backing SubscriptionEvent sees every push,
        // regardless of type.
        assert_eq!(umbrella.try_recv().unwrap()["type"], "ledgerClosed");
        assert_eq!(umbrella.try_recv().unwrap()["type"], "transaction");
    }

    /// A push message whose `"type"` has no registered stream is dropped
    /// (after reaching the umbrella channel) rather than causing the
    /// connection to close.
    #[test]
    fn push_with_unregistered_message_type_does_not_break_connection() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        let flow = handler
            .on_response(json!({ "type": "bookChanges", "ledger_index": 1 }));

        assert!(flow.is_continue());
    }

    /// Once a subscription's receiver is dropped, the next push of a matching
    /// message type must prune it from `subscriptions` - no manual GC pass
    /// required, and no unbounded growth of stale entries or upstream
    /// subscriptions.
    #[test]
    fn dropped_subscriber_is_pruned_from_subscriptions_on_next_matching_push() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        // Two subscriptions of the SAME message_type.
        let ack_a = acknowledge(&mut handler, "ledgerClosed");
        let ack_b = acknowledge(&mut handler, "ledgerClosed");
        let mut rx_b = ack_b.receiver;
        drop(ack_a.receiver); // only A's receiver is dropped

        assert_eq!(handler.subscriptions.len(), 2);

        let _ = handler.on_response(
            json!({ "type": "ledgerClosed", "ledger_index": 100 }),
        );

        assert_eq!(
            handler.subscriptions.len(),
            1,
            "the dropped subscription must be pruned on the next matching push"
        );

        // The surviving subscription's receiver still gets the message,
        // proving per-subscription channels are fully independent.
        let msg = rx_b
            .try_recv()
            .expect("surviving subscription must still receive the push");
        assert_eq!(msg["type"], "ledgerClosed");
    }

    /// Two subscriptions of the same `message_type` no longer share a
    /// channel: dropping one's receiver must not affect delivery to the
    /// other, even before any push triggers a pruning pass.
    #[test]
    fn same_message_type_subscriptions_have_independent_channels() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        let ack_a = acknowledge(&mut handler, "ledgerClosed");
        let ack_b = acknowledge(&mut handler, "ledgerClosed");
        let mut rx_b = ack_b.receiver;
        drop(ack_a.receiver);

        // Send directly on B's own sender - no push routing or pruning
        // pass involved, just proving the channels are independent.
        let sub_b = handler
            .subscriptions
            .values()
            .find(|sub| sub.sender.receiver_count() > 0)
            .expect("subscription B's channel must still have its receiver");
        let _ = sub_b
            .sender
            .send(json!({ "type": "ledgerClosed", "ledger_index": 42 }));

        let msg = rx_b
            .try_recv()
            .expect("subscription B must receive independently of A");
        assert_eq!(msg["ledger_index"], 42);
    }

    /// Unsubscribing rebuilds the wire payload from the original `subscribe`
    /// request's own fields (here `streams`) and removes the local
    /// bookkeeping entry immediately, rather than waiting for the next push
    /// to lazily prune it.
    #[test]
    fn unsubscribe_resends_original_fields_and_removes_subscription_immediately()
     {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        let (responder, mut sub_rx) = oneshot::channel();
        let payload = handler
            .on_request(SubscriptionSessionRequest::Subscribe(SubscribeRequest {
                request: json!({ "command": "subscribe", "streams": ["ledger"] }),
                responder,
                message_type: "ledgerClosed",
            }))
            .unwrap();
        let id = payload["id"].as_u64().unwrap();
        let _ = handler.on_response(json!({ "id": id, "status": "success" }));
        let _ = sub_rx.try_recv().unwrap().unwrap();

        assert_eq!(handler.subscriptions.len(), 1);

        let (unsub_responder, mut unsub_rx) = oneshot::channel();
        let unsub_payload = handler
            .on_request(SubscriptionSessionRequest::Unsubscribe(
                UnsubscribeRequest { id, responder: unsub_responder },
            ))
            .expect(
                "an active subscription must produce an unsubscribe payload",
            );

        assert_eq!(unsub_payload["command"], "unsubscribe");
        assert_eq!(unsub_payload["streams"], json!(["ledger"]));
        assert!(
            handler.subscriptions.is_empty(),
            "the subscription must be removed immediately, not lazily on next push"
        );

        let unsub_id = unsub_payload["id"].as_u64().unwrap();
        let _ =
            handler.on_response(json!({ "id": unsub_id, "status": "success" }));
        unsub_rx
            .try_recv()
            .expect("unsubscribe ack must be delivered synchronously")
            .expect("unsubscribe ack must not be an error");
    }

    /// Unsubscribing an id that is no longer tracked (already pruned, or a
    /// redundant second call) resolves locally without sending anything over
    /// the wire - there is nothing left to tell the server to stop.
    #[test]
    fn unsubscribe_on_unknown_id_resolves_locally_without_sending() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        let (responder, mut rx) = oneshot::channel();
        let sent = handler.on_request(SubscriptionSessionRequest::Unsubscribe(
            UnsubscribeRequest { id: 999, responder },
        ));

        assert!(sent.is_none(), "handler must choose not to send anything");
        rx.try_recv()
            .expect("must resolve synchronously")
            .expect("must resolve successfully, not as an error");
    }

    /// Unsubscribing one of two subscriptions for a *different* topic must
    /// not disturb the other's bookkeeping or delivery.
    #[test]
    fn unsubscribing_one_subscription_leaves_an_unrelated_one_intact() {
        let (stream_tx, _stream_rx) = broadcast::channel(16);
        let mut handler = SubscriptionHandler::new(stream_tx, 16);

        let ledger_ack = acknowledge(&mut handler, "ledgerClosed");
        let tx_ack = acknowledge(&mut handler, "transaction");
        let mut tx_rx = tx_ack.receiver;

        let (unsub_responder, _unsub_rx) = oneshot::channel();
        let _ = handler.on_request(SubscriptionSessionRequest::Unsubscribe(
            UnsubscribeRequest {
                id: ledger_ack.id,
                responder: unsub_responder,
            },
        ));

        assert_eq!(handler.subscriptions.len(), 1);
        let _ = handler
            .on_response(json!({ "type": "transaction", "hash": "ABC" }));
        assert_eq!(tx_rx.try_recv().unwrap()["hash"], "ABC");
    }
}
