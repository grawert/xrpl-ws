/// Request and response types for the `account_channels` command.
pub mod account_channels;
/// Request and response types for the `account_currencies` command.
pub mod account_currencies;
/// Request and response types for the `account_info` command.
pub mod account_info;
/// Request and response types for the `account_lines` command.
pub mod account_lines;
/// Request and response types for the `account_nfts` command.
pub mod account_nfts;
/// Request and response types for the `account_objects` command.
pub mod account_objects;
/// Request and response types for the `account_offers` command.
pub mod account_offers;
/// Request and response types for the `account_tx` command.
pub mod account_tx;
/// Request and response types for the `amm_info` command.
pub mod amm_info;
/// Request and response types for the `book_offers` command.
pub mod book_offers;
/// Request and response types for the `fee` command.
pub mod fee;
/// Request and response types for the `ledger` command.
pub mod ledger;
/// Request and response types for the `ledger_closed` command.
pub mod ledger_closed;
/// Request and response types for the `ledger_current` command.
pub mod ledger_current;
/// Request and response types for the `ledger_data` command.
pub mod ledger_data;
/// Request and response types for the `ledger_entry` command.
pub mod ledger_entry;
/// Request and response types for the `nft_buy_offers` command.
pub mod nft_buy_offers;
/// Request and response types for the `nft_sell_offers` command.
pub mod nft_sell_offers;
/// Request and response types for the `ripple_path_find` command.
pub mod ripple_path_find;
/// Request and response types for the `server_info` command.
pub mod server_info;
/// Request and response types for the `server_state` command.
pub mod server_state;
/// Request and response types for the `submit` command.
pub mod submit;
/// Request and response types for the `submit_multisigned` command.
pub mod submit_multisigned;
/// Request and response types for the `transaction_entry` command.
pub mod transaction_entry;
/// Request and response types for the `tx` command.
pub mod tx;

use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::error::XrplError;

/// Implemented by every XRPL request type, providing the command name, API version,
/// and a uniform way to serialize the request into a JSON value ready for submission
/// over a WebSocket or JSON-RPC connection.
///
/// # Examples
///
/// ```rust
/// use xrpl::request::XrplRequest;
/// use xrpl::request::account_info::AccountInfoRequest;
///
/// let req = AccountInfoRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
/// let json = req.to_value();
/// assert_eq!(json["command"], "account_info");
/// ```
pub trait XrplRequest: Serialize {
    /// Deserialized server response type for this request.
    type Response: Debug + DeserializeOwned;
    /// XRPL WebSocket/JSON-RPC command name (e.g. `"account_info"`).
    const COMMAND: &str;
    /// XRPL API version sent with every request; defaults to `2`.
    const API_VERSION: u32 = 2;

    /// Serializes the request into a [`serde_json::Value`] with `command` and
    /// `api_version` fields injected, ready for transmission to a rippled node.
    fn to_value(&self) -> Value {
        let mut map = serde_json::to_value(self)
            .expect("XrplRequest must be serializable")
            .as_object()
            .cloned()
            .unwrap_or_default();

        map.insert("command".into(), Self::COMMAND.into());
        map.insert("api_version".into(), Self::API_VERSION.into());
        map.into()
    }
}

/// Extends [`XrplRequest`] for commands that open a persistent subscription and push
/// server-initiated messages (e.g. `subscribe` for ledger or transaction streams).
///
/// The associated `Message` type is the deserialized form of each pushed event.
///
/// Subscriptions that use the `streams` wire field (e.g. `ledger`, `validations`,
/// `consensus`) should override `STREAM` with their stream name. Subscriptions
/// that use other fields (e.g. `accounts`, `books`) leave it at the default.
pub trait XrplSubscription: XrplRequest {
    /// The type of each streaming message delivered after the subscription is opened.
    type Message: Clone + Debug + Send + DeserializeOwned + 'static;
    /// XRPL stream name sent in the `streams` array (e.g. `"ledger"`).
    /// Leave at the default for subscriptions that use other request fields.
    const STREAM: &'static str = "";
}

/// Top-level envelope for every response from the rippled server.
///
/// Deserializes into either [`XrplResponse::Success`] (carrying the typed result) or
/// [`XrplResponse::Error`] (carrying the XRPL error code and message). Use
/// [`XrplResponse::result`] to convert into a standard [`Result`].
///
/// # Examples
///
/// ```rust
/// use xrpl::request::XrplResponse;
/// use xrpl::request::account_info::AccountInfoResponse;
///
/// async fn handle(resp: XrplResponse<AccountInfoResponse>) {
///     match resp.result() {
///         Ok(info) => println!("{}", info.account_data.balance),
///         Err(e) => eprintln!("XRPL error: {e}"),
///     }
/// }
/// ```
#[skip_serializing_none]
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum XrplResponse<T> {
    /// The server processed the request successfully; `result` holds the typed payload.
    Success {
        /// Optional correlation id echoed from the request.
        id: Option<Value>,
        /// Typed response payload.
        result: T,
        /// Response type label (e.g. `"response"`); wire field `type`.
        #[serde(rename = "type")]
        kind: String,
        /// Always `"success"` for this variant.
        status: String,
    },
    /// The server returned an error; inspect `error` and `error_message` for details.
    Error {
        /// Optional correlation id echoed from the request.
        id: Option<Value>,
        /// Short XRPL error token (e.g. `"invalidParams"`).
        error: String,
        /// Numeric XRPL error code.
        error_code: Option<i32>,
        /// Human-readable error description.
        error_message: Option<String>,
        /// Echo of the original request that triggered the error.
        request: Option<Value>,
        /// Response type label; wire field `type`.
        #[serde(rename = "type")]
        kind: String,
        /// Always `"error"` for this variant.
        status: String,
    },
}

impl<T> XrplResponse<T> {
    /// Converts this response envelope into a [`Result`], returning the typed
    /// payload on success or an [`XrplError::ApiError`] on failure.
    pub fn result(self) -> Result<T, XrplError> {
        match self {
            XrplResponse::Success { result, .. } => Ok(result),
            XrplResponse::Error {
                error, error_code, error_message, ..
            } => Err(XrplError::ApiError { error, error_code, error_message }),
        }
    }
}
