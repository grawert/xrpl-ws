pub mod account_channels;
pub mod account_currencies;
pub mod account_info;
pub mod account_lines;
pub mod account_nfts;
pub mod account_objects;
pub mod account_offers;
pub mod account_tx;
pub mod server_info;
pub mod submit;
pub mod tx;

use std::fmt::Debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::error::XrplError;

pub trait XrplRequest: Serialize {
    type Response: Debug + DeserializeOwned;
    const COMMAND: &'static str;
    const API_VERSION: u32 = 2;

    fn to_value(&self) -> Value {
        let mut map = serde_json::to_value(self)
            .expect("XrplRequest must be serializable")
            .as_object()
            .cloned()
            .unwrap_or_default();

        map.insert("id".into(), Uuid::new_v4().to_string().into());
        map.insert("command".into(), Self::COMMAND.into());
        map.insert("api_version".into(), Self::API_VERSION.into());
        map.into()
    }
}

pub trait XrplSubscription: XrplRequest {
    type Message: Clone + Debug + Send + DeserializeOwned + 'static;
    fn message_type() -> &'static str;
}

#[skip_serializing_none]
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum XrplResponse<T> {
    Success {
        id: Option<Value>,
        result: T,
        #[serde(rename = "type")]
        kind: String,
        status: String,
    },
    Error {
        id: Option<Value>,
        error: String,
        error_code: Option<i32>,
        error_message: Option<String>,
        request: Option<Value>,
        #[serde(rename = "type")]
        kind: String,
        status: String,
    },
}

impl<T> XrplResponse<T> {
    #[allow(clippy::result_large_err)]
    pub fn result(self) -> Result<T, XrplError> {
        match self {
            XrplResponse::Success { result, .. } => Ok(result),
            XrplResponse::Error {
                error, error_code, error_message, ..
            } => Err(XrplError::ApiError { error, error_code, error_message }),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, XrplResponse::Success { .. })
    }
}
