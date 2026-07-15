# Crate Documentation

**Version:** 0.1.0

**Format Version:** 60

# Module `xrpl`

# XRPL Client Library

Lightweight async WebSocket client for the XRP Ledger. Supports requests,
subscriptions, and automatic reconnection. Transaction signing and
serialization is delegated to external libraries.

## Installation

```toml
[dependencies]
xrpl-ws = "0.1"
```

## Requests

```no_run
use xrpl::{Client, request::account_info::AccountInfoRequest};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("wss://xrplcluster.com");

    let request = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");

    let response = client.request(&request).await?;
    println!("Account balance: {}", response.result()?.account_data.balance);

    Ok(())
}
```

## Subscriptions

Use [`Client::subscription`] to open a shared connection, then call
[`SubscriptionSession::subscribe`] to receive a stream of validated
transactions for a specific account. After each transaction,
[`util::available_balance`] returns the spendable balance after reserves.

```no_run
use xrpl::{Client, subscriptions::AccountTransactionsSubscription, util::available_balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";

    let sub = AccountTransactionsSubscription::validated([account])?;
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;

    while let Ok(tx) = stream.recv().await {
        let balance = available_balance(&client, account).await?;
        println!("{} - {} - spendable: {} drops", tx.hash, tx.engine_result, balance);
    }

    Ok(())
}
```

When processing incoming payments, always use [`types::HasTransactionMeta::delivered_amount`]
instead of the transaction's `Amount` field to guard against partial-payment attacks:

```no_run
use xrpl::{Client, subscriptions::AccountTransactionsSubscription};
use xrpl::types::HasTransactionMeta;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let account = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";

    let sub = AccountTransactionsSubscription::validated([account])?;
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;

    while let Ok(tx) = stream.recv().await {
        if !tx.validated { continue; }
        if let Some(amount) = tx.delivered_amount() {
            println!("Received: {amount}");
        }
    }

    Ok(())
}
```

### `session.recv()` vs `stream.recv()`

[`SubscriptionSession::subscribe`] returns a [`SubscriptionStream`] scoped
to that subscription's wire message type, deserialized into the concrete
type (e.g. [`subscriptions::LedgerMessage`]). This separates *different*
message types from each other, but not multiple subscriptions of the
*same* type: rippled tags a pushed message with a type, not a
subscription id, so two [`subscriptions::AccountTransactionsSubscription`]s
for different accounts on one session both receive every `"transaction"`
push, not just their own account's. Match on the message content to tell
them apart:

```no_run
use xrpl::{Client, subscriptions::AccountTransactionsSubscription};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let account = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session
        .subscribe(&AccountTransactionsSubscription::validated([account])?)
        .await?;

    while let Ok(msg) = stream.recv().await {
        match msg.tx_json.account.as_str() {
            a if a == account => println!("sent by {account}: {}", msg.hash),
            // Some other account's traffic, not handled here
            _ => {}
        }
    }
    Ok(())
}
```

The session itself also has [`SubscriptionSession::recv`], which reads from
a single unified channel carrying every message pushed over the shared
connection - regardless of which or how many subscriptions are open on
it - typed as the [`SubscriptionEvent`] enum. Reach for it when one loop
needs to react to several subscription types together; reach for the
typed stream when a task only cares about one.

```no_run
use xrpl::{Client, SubscriptionEvent};
use xrpl::subscriptions::{LedgerSubscription, TransactionsSubscription};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;

    // Each subscribe() call opens its own typed stream over the same
    // shared connection; both streams are kept alive here so their
    // subscriptions survive a reconnect.
    let (_, mut ledgers) = session.subscribe(&LedgerSubscription::new()).await?;
    let (_, _txs) = session.subscribe(&TransactionsSubscription::validated()).await?;

    // Typed stream: only ledger messages, already deserialized.
    tokio::spawn(async move {
        while let Ok(msg) = ledgers.recv().await {
            println!("[ledgers] {} closed", msg.ledger_index);
        }
    });

    // Unified session: every message pushed on the connection, tagged by type.
    while let Ok(event) = session.recv().await {
        match event {
            SubscriptionEvent::Ledger(msg) => {
                println!("[session] ledger {} closed", msg.ledger_index);
            }
            SubscriptionEvent::Transaction(tx) => {
                println!("[session] tx {} ({})", tx.hash, tx.engine_result);
            }
            SubscriptionEvent::BookChanges(_) => {}
            // A push whose "type" isn't modelled above - most likely a
            // stream this library doesn't implement yet.
            SubscriptionEvent::Unknown { message_type, .. } => {
                eprintln!("[session] unrecognized push type: {message_type}");
            }
            _ => {}
        }
    }
    Ok(())
}
```

### Ending a subscription

Dropping a [`SubscriptionStream`] stops it locally and - best-effort,
fire-and-forget - tells the server to stop pushing. To wait for the
server's acknowledgement (and catch a protocol-level failure), call
[`SubscriptionStream::unsubscribe`] instead:

```no_run
use xrpl::{Client, subscriptions::LedgerSubscription};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;
    let (_resp, mut stream) =
        session.subscribe(&LedgerSubscription::new()).await?;

    let msg = stream.recv().await?;
    println!("ledger {} closed", msg.ledger_index);

    stream.unsubscribe().await?;
    Ok(())
}
```

A [`SubscriptionStream`] is independently owned and keeps working even
after the [`SubscriptionSession`] that created it is dropped - only
`unsubscribe()`, or dropping the stream itself, ends it.

## Builders

Use the builders in [`types::builders`] to construct transaction payloads.
Call `.fill(&client)` before `.build()` to auto-populate `Sequence`, `Fee`,
and `LastLedgerSequence` from the network.

```no_run
use xrpl::{Client, xrp, types::{PaymentFlag, builders::PaymentBuilder}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");

    let payment = PaymentBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        xrp!(1.99),
    )
    .with_flags(PaymentFlag::PartialPayment)
    .fill(&client)
    .await?
    .build()?;

    Ok(())
}
```

Transactions with time-based fields use the Ripple epoch (seconds since
2000-01-01 UTC). Use [`time::ripple_now`] to avoid off-by-30-years errors:

```no_run
use xrpl::{Client, xrp, time::ripple_now, types::builders::CheckCreateBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");

    let check = CheckCreateBuilder::new(
        "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh",
        "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
        xrp!(1.99),
    )
    .with_expiration(ripple_now() + 86_400) // expires in 24 hours
    .fill(&client)
    .await?
    .build()?;

    Ok(())
}
```

See [`types::builders`] for the full list of available transaction builders.

## Signing

Signing and binary serialization are outside the scope of this library and
are intentionally delegated to purpose-built crates (e.g. `ripple-keypairs`,
`xrpl-mithril`). Implement the [`types::SigningContext`] trait on your wallet
type to bridge the two.

The process follows the XRPL signing protocol: serialize the transaction to
binary (excluding the signature fields), prepend `HASH_PREFIX_TRANSACTION_SIGN`
(the "STX" prefix), sign the bytes, attach the signature, then serialize the
final blob for submission.

```ignore
use anyhow::Context;
use ripple_keypairs::{PrivateKey, PublicKey};
use xrpl_mithril::codec::serializer::serialize_json_object;
use xrpl_mithril::codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
use xrpl::types::{Transaction, SigningContext};

struct Wallet {
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl SigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_transaction(&self, tx: &Transaction) -> Result<String, Self::Error> {
        let mut tx_json = serde_json::to_value(tx)?;
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let buf = {
            let map = tx_json.as_object().context("Transaction should be JSON object")?;
            let mut buf = Vec::new();
            serialize_json_object(map, &mut buf, true)?;
            buf
        };

        let mut signing_bytes = Vec::with_capacity(4 + buf.len());
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&buf);
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let map = tx_json.as_object().context("Transaction should be JSON object")?;
        let mut buf = Vec::new();
        serialize_json_object(map, &mut buf, false)?;

        Ok(hex::encode(buf).to_uppercase())
    }
}
```

Once the wallet is wired up, pass it to [`types::builders::SubmitRequestBuilder`]
together with the built transaction. Signing happens inside `build()` and the
result goes straight to [`Client::request`]:

```ignore
use xrpl::{Client, xrp, types::builders::{PaymentBuilder, SubmitRequestBuilder}};

let client = Client::new("wss://xrplcluster.com");
let wallet = Wallet { /* ... */ };

let tx = PaymentBuilder::new(
    wallet.public_key.derive_address(),
    "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe",
    xrp!(10),
)
.fill(&client)
.await?
.build()?;

let req = SubmitRequestBuilder::new(&tx, &wallet).build()?;
let result = client.request(&req).await?;

assert_eq!(result.result()?.engine_result, "tesSUCCESS");
```

## Modules

## Module `config`

Client configuration (timeouts, channel sizes, reconnect backoff).

```rust
pub mod config { /* ... */ }
```

### Types

#### Struct `ClientConfig`

Configuration for XRPL WebSocket client behavior.

## Timeouts

`request_timeout` - deadline for a single rippled response before
[`crate::error::XrplError::Timeout`] is returned (default: 30 s).

`keepalive_interval` - interval between WebSocket ping frames sent to the
server to keep the connection alive (default: 20 s).

## Reconnection backoff

On disconnect the client reconnects with exponential backoff: `initial_backoff`
after the first failure, doubled each attempt up to `max_backoff`.

## Channel sizes

`cmd_channel_size` - buffer depth for outgoing requests (default: 32).

`subscription_channel_size` - buffer depth for incoming subscription messages
before backpressure is applied (default: 32).

# Example

```rust
use xrpl::{Client, ClientConfig};

#[tokio::main]
async fn main() -> Result<(), xrpl::XrplError> {
    let config = ClientConfig::default()
        .with_request_timeout_secs(60)
        .with_keepalive_secs(30);
    let client = Client::with_config("wss://xrplcluster.com", config);
    Ok(())
}
```

```rust
pub struct ClientConfig {
    pub cmd_channel_size: usize,
    pub subscription_channel_size: usize,
    pub request_timeout: std::time::Duration,
    pub keepalive_interval: std::time::Duration,
    pub initial_backoff: std::time::Duration,
    pub max_backoff: std::time::Duration,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `cmd_channel_size` | `usize` | Size of the command channel buffer (default: 32) |
| `subscription_channel_size` | `usize` | Size of the subscription channel buffer (default: 32) |
| `request_timeout` | `std::time::Duration` | Request timeout (default: 30 seconds) |
| `keepalive_interval` | `std::time::Duration` | Keepalive ping interval (default: 20 seconds) |
| `initial_backoff` | `std::time::Duration` | Initial backoff duration for reconnection attempts (default: 1 second) |
| `max_backoff` | `std::time::Duration` | Maximum backoff duration for reconnection attempts (default: 30 seconds) |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Create a new ClientConfig with default values

- ```rust
  pub fn with_request_timeout(self: Self, timeout: Duration) -> Self { /* ... */ }
  ```
  Set the request timeout

- ```rust
  pub fn with_request_timeout_secs(self: Self, timeout_secs: u64) -> Self { /* ... */ }
  ```
  Set the request timeout in seconds (convenience method)

- ```rust
  pub fn with_subscription_channel_size(self: Self, size: usize) -> Self { /* ... */ }
  ```
  Set the subscription channel size

- ```rust
  pub fn with_cmd_channel_size(self: Self, size: usize) -> Self { /* ... */ }
  ```
  Set the command channel size

- ```rust
  pub fn with_keepalive_interval(self: Self, interval: Duration) -> Self { /* ... */ }
  ```
  Set the keepalive interval

- ```rust
  pub fn with_keepalive_secs(self: Self, interval_secs: u64) -> Self { /* ... */ }
  ```
  Set the keepalive interval in seconds (convenience method)

- ```rust
  pub fn with_initial_backoff(self: Self, backoff: Duration) -> Self { /* ... */ }
  ```
  Set the initial backoff duration

- ```rust
  pub fn with_initial_backoff_secs(self: Self, backoff_secs: u64) -> Self { /* ... */ }
  ```
  Set the initial backoff duration in seconds (convenience method)

- ```rust
  pub fn with_max_backoff(self: Self, backoff: Duration) -> Self { /* ... */ }
  ```
  Set the maximum backoff duration

- ```rust
  pub fn with_max_backoff_secs(self: Self, backoff_secs: u64) -> Self { /* ... */ }
  ```
  Set the maximum backoff duration in seconds (convenience method)

###### Trait Implementations

## Module `error`

Error types returned by the client.

```rust
pub mod error { /* ... */ }
```

### Types

#### Enum `XrplError`

Errors that can be returned by the XRPL WebSocket client.

# Examples

```no_run
use xrpl::{Client, XrplError};
use xrpl::request::account_info::AccountInfoRequest;

#[tokio::main]
async fn main() {
    let client = Client::new("wss://xrplcluster.com");
    let req = AccountInfoRequest::new("rBadAccount");
    match client.request(&req).await {
        Err(XrplError::ApiError { error, error_code, .. }) => {
            eprintln!("rippled error {error} (code {error_code:?})");
        }
        Err(XrplError::Timeout(ms)) => eprintln!("timed out after {ms}ms"),
        _ => {}
    }
}
```

```rust
pub enum XrplError {
    ConnectionError(String),
    Disconnected,
    Timeout(u64),
    ParseError(String),
    SerializeError(String),
    MessageDropped(u64),
    ApiError {
        error: String,
        error_code: Option<i32>,
        error_message: Option<String>,
    },
}
```

##### Variants

###### `ConnectionError`

The WebSocket connection to the node could not be established.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `Disconnected`

The WebSocket connection was closed before the operation completed.

###### `Timeout`

No response was received within the configured timeout period (milliseconds).

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u64` |  |

###### `ParseError`

The server response could not be deserialized into the expected type.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `SerializeError`

The request could not be serialized into JSON.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `MessageDropped`

The subscription channel fell behind and messages were dropped.
The subscription is still active - call [`crate::SubscriptionSession::recv`] again to continue.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u64` |  |

###### `ApiError`

The rippled node returned an application-level error.

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `error` | `String` | Short error name returned by rippled (e.g. `"actNotFound"`). |
| `error_code` | `Option<i32>` | Numeric rippled error code (e.g. `23` for `actNotFound`), when present. |
| `error_message` | `Option<String>` | Human-readable description of the error, when present. Populated from<br>`error_message` in the response, falling back to `error_exception` for<br>internal rippled errors (neither field is in the official spec, but both<br>are sent in practice). |

##### Implementations

###### Methods

- ```rust
  pub fn error_code(self: &Self) -> Option<i32> { /* ... */ }
  ```
  Numeric rippled error code, when the error originated from an API-level response.

- ```rust
  pub fn error_message(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Human-readable description of the error, when available.

###### Trait Implementations

- **Display**
  - ```rust
    fn fmt(self: &Self, __formatter: &mut ::core::fmt::Formatter<''_>) -> ::core::fmt::Result { /* ... */ }
    ```

- **Error**
## Module `request`

Request types and response envelopes for all XRPL JSON-RPC commands.

```rust
pub mod request { /* ... */ }
```

### Modules

## Module `account_channels`

Request and response types for the `account_channels` command.

```rust
pub mod account_channels { /* ... */ }
```

### Types

#### Struct `AccountChannelsRequest`

Retrieves all open payment channels where the specified account is the source.

Use this to inspect how much XRP an account has allocated across its outbound
payment channels and what each channel's current balance is.

# Examples

```rust
use xrpl::request::account_channels::AccountChannelsRequest;

let req = AccountChannelsRequest { limit: Some(50), ..AccountChannelsRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
```

```rust
pub struct AccountChannelsRequest {
    pub account: String,
    pub destination_account: Option<String>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Source account whose outbound channels are queried (r-address). |
| `destination_account` | `Option<String>` | Restrict results to channels whose destination is this account (r-address). |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `limit` | `Option<u32>` | Maximum number of channels to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given source account.

- ```rust
  pub fn with_destination_account</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the destination account to filter channels by.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of channels to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountChannelsResponse`

Response payload for an [`AccountChannelsRequest`].

Contains the list of open payment channels owned by the queried account
along with ledger context and pagination state.

# Examples

```rust
use xrpl::request::account_channels::AccountChannelsResponse;

fn print_totals(resp: &AccountChannelsResponse) {
    for ch in &resp.channels {
        println!("channel {} - allocated: {} drops", ch.channel_id, ch.amount);
    }
}
```

```rust
pub struct AccountChannelsResponse {
    pub account: String,
    pub channels: Vec<AccountChannel>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
    pub limit: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Source account address (r-address) whose channels are returned. |
| `channels` | `Vec<AccountChannel>` | List of open payment channels sourced from `account`. |
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger used to answer the request. |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more channels remain on the next page. |
| `limit` | `Option<u32>` | Effective page size applied by the server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountChannel`

A single unidirectional XRP payment channel.

Represents a channel opened by the source account that allows off-ledger
micro-payments to the destination, settling on-ledger via claims.

# Examples

```rust
use xrpl::request::account_channels::AccountChannel;

fn available_drops(ch: &AccountChannel) -> u64 {
    ch.amount.parse::<u64>().unwrap_or(0)
        .saturating_sub(ch.balance.parse::<u64>().unwrap_or(0))
}
```

```rust
pub struct AccountChannel {
    pub account: String,
    pub amount: String,
    pub balance: String,
    pub channel_id: String,
    pub destination_account: String,
    pub settle_delay: u32,
    pub public_key: Option<String>,
    pub public_key_hex: Option<String>,
    pub expiration: Option<u32>,
    pub cancel_after: Option<u32>,
    pub source_tag: Option<u32>,
    pub destination_tag: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Source account address (r-address) that funded the channel. |
| `amount` | `String` | Total XRP (in drops) allocated to the channel. |
| `balance` | `String` | XRP (in drops) already claimed by the destination. |
| `channel_id` | `String` | Unique 64-hex-character channel identifier. |
| `destination_account` | `String` | Destination account that can claim XRP from the channel (r-address). |
| `settle_delay` | `u32` | Minimum seconds the source must wait to close the channel after requesting closure. |
| `public_key` | `Option<String>` | Source's secp256k1 public key for signing channel claims (base58). |
| `public_key_hex` | `Option<String>` | Source's public key in hex format. |
| `expiration` | `Option<u32>` | Ripple epoch timestamp when the channel expires (mutable, set by source). |
| `cancel_after` | `Option<u32>` | Ripple epoch timestamp after which anyone can close the channel (immutable). |
| `source_tag` | `Option<u32>` | Source-defined tag for routing or reference. |
| `destination_tag` | `Option<u32>` | Destination-defined tag for routing or reference. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_currencies`

Request and response types for the `account_currencies` command.

```rust
pub mod account_currencies { /* ... */ }
```

### Types

#### Struct `AccountCurrenciesRequest`

Retrieves the set of currencies an account can send or receive via trust lines.

Useful for building currency selectors in wallet UIs or verifying that a trust
line exists before issuing a payment in a specific currency.

# Examples

```rust
use xrpl::request::account_currencies::AccountCurrenciesRequest;

let req = AccountCurrenciesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
```

```rust
pub struct AccountCurrenciesRequest {
    pub account: String,
    pub strict: Option<bool>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose supported currencies are queried (r-address). |
| `strict` | `Option<bool>` | If `true`, requires the `account` to be a classic address or public key. |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account.

- ```rust
  pub fn with_strict(self: Self, strict: bool) -> Self { /* ... */ }
  ```
  Requires the account to be a classic address or public key.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the target ledger by its hash.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut (e.g. `"validated"`, `"current"`).

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountCurrenciesResponse`

Response payload for an [`AccountCurrenciesRequest`].

Lists all currency codes the account can currently send or receive, derived
from its active trust lines in the specified ledger.

# Examples

```rust
use xrpl::request::account_currencies::AccountCurrenciesResponse;

fn can_receive_usd(resp: &AccountCurrenciesResponse) -> bool {
    resp.receive_currencies.iter().any(|c| c == "USD")
}
```

```rust
pub struct AccountCurrenciesResponse {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub receive_currencies: Vec<String>,
    pub send_currencies: Vec<String>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger used to answer the request. |
| `receive_currencies` | `Vec<String>` | Currency codes the account can receive (3-char ISO or 40-hex non-standard). |
| `send_currencies` | `Vec<String>` | Currency codes the account can send (3-char ISO or 40-hex non-standard). |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_info`

Request and response types for the `account_info` command.

```rust
pub mod account_info { /* ... */ }
```

### Types

#### Struct `AccountInfoRequest`

Retrieves core account state: XRP balance, sequence number, flags, and owner count.

Optionally includes queued transactions and signer lists. This is the primary
request for checking whether an account is funded and for reading its current
sequence number before building a transaction.

# Examples

```rust
use xrpl::request::account_info::AccountInfoRequest;

let req = AccountInfoRequest { queue: Some(true), ..AccountInfoRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
```

```rust
pub struct AccountInfoRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub queue: Option<bool>,
    pub signer_lists: Option<bool>,
    pub strict: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account to look up (r-address, base58check encoded). |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `queue` | `Option<bool>` | When `true`, include queued transaction data in the response. |
| `signer_lists` | `Option<bool>` | When `true`, include the account's signer lists in the response. |
| `strict` | `Option<bool>` | When `true`, only accept a fully-canonical account address (no aliases). |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account address.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the target ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_queue(self: Self, queue: bool) -> Self { /* ... */ }
  ```
  Configures whether to include queued transaction data.

- ```rust
  pub fn with_signer_lists(self: Self, signer_lists: bool) -> Self { /* ... */ }
  ```
  Configures whether to include signer lists.

- ```rust
  pub fn with_strict(self: Self, strict: bool) -> Self { /* ... */ }
  ```
  Configures whether to reject non-canonical account addresses.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountInfoResponse`

Response payload for an [`AccountInfoRequest`].

Provides the on-ledger state of the account, including its [`AccountRoot`] object.

# Examples

```rust
use xrpl::request::account_info::AccountInfoResponse;

fn next_sequence(resp: &AccountInfoResponse) -> u32 {
    resp.account_data.sequence
}
```

```rust
pub struct AccountInfoResponse {
    pub account_data: AccountRoot,
    pub signer_lists: Option<Vec<String>>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub queue_data: Option<QueueData>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account_data` | `AccountRoot` | The account's ledger object containing balance, flags, and sequence number. |
| `signer_lists` | `Option<Vec<String>>` | Signer lists attached to the account; populated when `signer_lists` was `true`. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when querying the open ledger). |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger used to answer the request. |
| `queue_data` | `Option<QueueData>` | Queued transaction summary; populated when `queue` was `true`. |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountRoot`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

The on-ledger `AccountRoot` object for an XRPL account.

Holds the authoritative state of the account as recorded in a specific ledger:
XRP balance, current sequence number, and owner count used to calculate reserves.
Fields are PascalCase on the wire (`Account`, `Balance`, `Flags`, ...).

# Examples

```rust
use xrpl::request::account_info::AccountRoot;

fn available_xrp_drops(root: &AccountRoot, reserve_drops: u64) -> u64 {
    root.balance.parse::<u64>().unwrap_or(0).saturating_sub(reserve_drops)
}
```

```rust
pub struct AccountRoot {
    pub account: String,
    pub balance: String,
    pub flags: crate::types::AccountFlags,
    pub ledger_entry_type: String,
    pub owner_count: u32,
    pub previous_txn_id: String,
    pub previous_txn_lgr_seq: u32,
    pub sequence: u32,
    pub index: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | XRPL account address (r-address, base58check encoded). Wire: `Account`. |
| `balance` | `String` | XRP balance in drops as a string. Wire: `Balance`. |
| `flags` | `crate::types::AccountFlags` | Active account flags. Wire: `Flags`. |
| `ledger_entry_type` | `String` | Always `"AccountRoot"`. Wire: `LedgerEntryType`. |
| `owner_count` | `u32` | Number of objects the account owns (affects reserve). Wire: `OwnerCount`. |
| `previous_txn_id` | `String` | Transaction ID of the last transaction that modified this account. Wire: `PreviousTxnID`. |
| `previous_txn_lgr_seq` | `u32` | Ledger sequence containing the last modifying transaction. Wire: `PreviousTxnLgrSeq`. |
| `sequence` | `u32` | Next valid sequence number for transactions from this account. Wire: `Sequence`. |
| `index` | `String` | Ledger object index (SHA-512Half of account ID). Wire: `index`. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `QueueData`

Summary of transactions queued for the account but not yet applied to a validated ledger.

Returned when [`AccountInfoRequest::queue`] is `true`. Useful for determining
the next available sequence number when submitting multiple transactions in quick
succession without waiting for each to validate.

# Examples

```rust
use xrpl::request::account_info::QueueData;

fn next_free_sequence(queue: &QueueData, current_seq: u32) -> u32 {
    queue.highest_sequence.map(|s| s + 1).unwrap_or(current_seq)
}
```

```rust
pub struct QueueData {
    pub txn_count: u32,
    pub auth_change_queued: Option<bool>,
    pub lowest_sequence: Option<u32>,
    pub highest_sequence: Option<u32>,
    pub max_spend_drops_total: Option<String>,
    pub transactions: Option<Vec<QueueTransaction>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `txn_count` | `u32` | Number of transactions currently in the queue for this account. |
| `auth_change_queued` | `Option<bool>` | `true` if any queued transaction would change the account's auth settings. |
| `lowest_sequence` | `Option<u32>` | Lowest sequence number among all queued transactions. |
| `highest_sequence` | `Option<u32>` | Highest sequence number among all queued transactions. |
| `max_spend_drops_total` | `Option<String>` | Maximum XRP (in drops) that the queued transactions could spend combined. |
| `transactions` | `Option<Vec<QueueTransaction>>` | Per-transaction details; present only when the server includes them. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `QueueTransaction`

Per-transaction detail within [`QueueData`].

Each entry describes one queued transaction and the resources it would consume
if applied to the ledger.

# Examples

```rust
use xrpl::request::account_info::QueueTransaction;

fn is_high_fee(tx: &QueueTransaction, threshold: u64) -> bool {
    tx.fee.parse::<u64>().unwrap_or(0) > threshold
}
```

```rust
pub struct QueueTransaction {
    pub auth_change: bool,
    pub fee: String,
    pub fee_level: String,
    pub max_spend_drops: String,
    pub seq: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `auth_change` | `bool` | `true` if this transaction would change the account's signing authority. |
| `fee` | `String` | Transaction fee in drops as a string. |
| `fee_level` | `String` | Fee level relative to the minimum fee (higher means higher priority). |
| `max_spend_drops` | `String` | Maximum XRP (in drops) this transaction could spend including the fee. |
| `seq` | `u32` | Sequence number of this queued transaction. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_lines`

Request and response types for the `account_lines` command.

```rust
pub mod account_lines { /* ... */ }
```

### Types

#### Struct `AccountLinesRequest`

Retrieves trust lines (IOU balances) for an account.

Returns the set of trust lines linking the account to other issuers, including
balances, limits, and rippling/freeze flags. Paginate with `limit` and `marker`
for accounts with many trust lines.

# Examples

```rust
use xrpl::request::account_lines::AccountLinesRequest;

let req = AccountLinesRequest { limit: Some(200), ..AccountLinesRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
```

```rust
pub struct AccountLinesRequest {
    pub account: String,
    pub ignore_default: Option<bool>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
    pub peer: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose trust lines are queried (r-address). |
| `ignore_default` | `Option<bool>` | When `true`, suppress trust lines that are in their default (zero-balance, default-limit) state. |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `limit` | `Option<u32>` | Maximum number of trust lines to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |
| `peer` | `Option<String>` | Restrict results to the trust line with this specific counterparty account (r-address). |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account address.

- ```rust
  pub fn with_ignore_default(self: Self, value: bool) -> Self { /* ... */ }
  ```
  Sets whether to ignore default trust lines.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of trust lines to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

- ```rust
  pub fn with_peer</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, peer: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the peer account to filter by.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountLinesResponse`

Response payload for an [`AccountLinesRequest`].

Contains the page of trust lines for the queried account along with ledger
context and a pagination marker for retrieving subsequent pages.

# Examples

```rust
use xrpl::request::account_lines::AccountLinesResponse;

fn usd_balance(resp: &AccountLinesResponse) -> Option<&str> {
    resp.lines.iter()
        .find(|l| l.currency == "USD")
        .map(|l| l.balance.as_str())
}
```

```rust
pub struct AccountLinesResponse {
    pub account: String,
    pub lines: Vec<Trustline>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
    pub limit: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose trust lines are returned (r-address). |
| `lines` | `Vec<Trustline>` | Trust lines for the account in the queried ledger. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when querying the open ledger). |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger used to answer the request. |
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more trust lines remain on the next page. |
| `limit` | `Option<u32>` | Effective page size applied by the server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Trustline`

A single trust line between two XRPL accounts for an issued currency.

Describes the bilateral agreement that allows an account to hold an IOU balance
from an issuer, including the current balance, trust limits, and rippling/freeze
flags on both sides of the line.

# Examples

```rust
use xrpl::request::account_lines::Trustline;

fn is_frozen(line: &Trustline) -> bool {
    line.freeze.unwrap_or(false) || line.freeze_peer.unwrap_or(false)
}
```

```rust
pub struct Trustline {
    pub account: String,
    pub balance: String,
    pub currency: String,
    pub limit: String,
    pub limit_peer: String,
    pub quality_in: u32,
    pub quality_out: u32,
    pub no_ripple: Option<bool>,
    pub no_ripple_peer: Option<bool>,
    pub authorized: Option<bool>,
    pub peer_authorized: Option<bool>,
    pub freeze: Option<bool>,
    pub freeze_peer: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Counterparty account address (r-address) on the other side of the trust line. |
| `balance` | `String` | Current IOU balance (positive = account holds, negative = account owes). |
| `currency` | `String` | Currency code (3-char ISO or 40-hex non-standard). |
| `limit` | `String` | Maximum IOU balance the account trusts the counterparty to owe. |
| `limit_peer` | `String` | Maximum IOU balance the counterparty trusts this account to owe. |
| `quality_in` | `u32` | Inbound quality (exchange rate multiplier) set by this account; 0 means 1:1. |
| `quality_out` | `u32` | Outbound quality (exchange rate multiplier) set by this account; 0 means 1:1. |
| `no_ripple` | `Option<bool>` | `true` if this account has set the NoRipple flag on this trust line. |
| `no_ripple_peer` | `Option<bool>` | `true` if the counterparty has set the NoRipple flag on this trust line. |
| `authorized` | `Option<bool>` | `true` if this account has authorized the counterparty's trust line. |
| `peer_authorized` | `Option<bool>` | `true` if the counterparty has authorized this account's trust line. |
| `freeze` | `Option<bool>` | `true` if this account has frozen the trust line. |
| `freeze_peer` | `Option<bool>` | `true` if the counterparty has frozen the trust line. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_nfts`

Request and response types for the `account_nfts` command.

```rust
pub mod account_nfts { /* ... */ }
```

### Types

#### Struct `AccountNftsRequest`

Retrieves NFTokens owned by an account (XLS-20).

Returns metadata for each NFToken the account currently holds, including
the token ID, issuer, taxon, and URI. Paginate with `limit` and `marker`
for accounts with large NFT collections.

# Examples

```rust
use xrpl::request::account_nfts::AccountNftsRequest;

let req = AccountNftsRequest { limit: Some(100), ..AccountNftsRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
```

```rust
pub struct AccountNftsRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose NFTokens are queried (r-address). |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `limit` | `Option<u32>` | Maximum number of NFTokens to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of NFTokens to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountNftsResponse`

Response payload for an [`AccountNftsRequest`].

Contains the page of NFTokens owned by the queried account along with ledger
context and a pagination marker for retrieving subsequent pages.

# Examples

```rust
use xrpl::request::account_nfts::AccountNftsResponse;

fn count_nfts(resp: &AccountNftsResponse) -> usize {
    resp.account_nfts.len()
}
```

```rust
pub struct AccountNftsResponse {
    pub account: String,
    pub account_nfts: Vec<AccountNFToken>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
    pub limit: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose NFTokens are returned (r-address). |
| `account_nfts` | `Vec<AccountNFToken>` | NFTokens currently owned by the account. |
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger used to answer the request. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when querying the open ledger). |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more NFTokens remain on the next page. |
| `limit` | `Option<u32>` | Effective page size applied by the server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountNFToken`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A single NFToken owned by an account (XLS-20).

Carries the immutable metadata of the token as recorded in the ledger.
Wire fields are PascalCase (`Flags`, `Issuer`, `NFTokenID`, ...).

# Examples

```rust
use xrpl::request::account_nfts::AccountNFToken;

fn is_transferable(token: &AccountNFToken) -> bool {
    // tfTransferable flag bit
    token.flags & 0x0008 != 0
}
```

```rust
pub struct AccountNFToken {
    pub flags: u32,
    pub issuer: String,
    pub nftoken_id: String,
    pub nftoken_taxon: u32,
    pub uri: Option<String>,
    pub nft_serial: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `flags` | `u32` | Bitfield of NFToken flags (e.g. `tfTransferable`, `tfOnlyXRP`). Wire: `Flags`. |
| `issuer` | `String` | Account that minted the token (r-address). Wire: `Issuer`. |
| `nftoken_id` | `String` | Unique 256-bit token identifier (64 hex chars). Wire: `NFTokenID`. |
| `nftoken_taxon` | `u32` | Issuer-defined taxon that groups related tokens. Wire: `NFTokenTaxon`. |
| `uri` | `Option<String>` | Hex-encoded URI pointing to the token's metadata (e.g. IPFS). Wire: `URI`. |
| `nft_serial` | `u32` | Per-issuer-taxon serial number assigned at mint time. Wire: `nft_serial`. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_objects`

Request and response types for the `account_objects` command.

```rust
pub mod account_objects { /* ... */ }
```

### Types

#### Struct `AccountObjectsRequest`

Retrieves all ledger objects owned by an account.

Covers any object type that counts against the account's owner reserve: offers,
escrows, payment channels, trust lines, signer lists, tickets, checks, and more.
Filter by type with `kind`, or set `deletion_blockers_only` to find objects that
prevent account deletion.

# Examples

```rust
use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};

let req = AccountObjectsRequest {
    kind: Some(AccountObjectType::Offer),
    limit: Some(100),
    ..AccountObjectsRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
};
```

```rust
pub struct AccountObjectsRequest {
    pub account: String,
    pub deletion_blockers_only: Option<bool>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
    pub kind: Option<AccountObjectType>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose owned objects are queried (r-address). |
| `deletion_blockers_only` | `Option<bool>` | When `true`, return only objects that block account deletion. |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `limit` | `Option<u32>` | Maximum number of objects to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |
| `kind` | `Option<AccountObjectType>` | Filter results to include only objects of this specific type. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account address.

- ```rust
  pub fn with_deletion_blockers_only(self: Self, value: bool) -> Self { /* ... */ }
  ```
  Sets whether to return only objects that block account deletion.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of objects to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

- ```rust
  pub fn with_kind(self: Self, kind: AccountObjectType) -> Self { /* ... */ }
  ```
  Sets the object type to filter by.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Enum `AccountObjectType`

**Attributes:**

- `Other("#[serde(rename_all = \"snake_case\")]")`

Filter for [`AccountObjectsRequest`] restricting results to one ledger object type.

Wire values are snake_case strings (e.g. `"offer"`, `"payment_channel"`).
Use this to narrow a query to only the object category you care about, reducing
the result set and avoiding unnecessary pagination.

# Examples

```rust
use xrpl::request::account_objects::{AccountObjectsRequest, AccountObjectType};

let req = AccountObjectsRequest {
    kind: Some(AccountObjectType::Escrow),
    ..AccountObjectsRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
};
```

```rust
pub enum AccountObjectType {
    Bridge,
    Check,
    Credential,
    Delegate,
    DepositPreauth,
    DID,
    Escrow,
    MPToken,
    MPTokenIssuance,
    NFTokenOffer,
    NFTokenPage,
    Offer,
    Oracle,
    PayChannel,
    PermissionedDomain,
    RippleState,
    SignerList,
    Ticket,
    XChainOwnedClaimId,
    XChainOwnedCreateAccountClaimId,
}
```

##### Variants

###### `Bridge`

Cross-chain bridge object.

###### `Check`

Check object (deferred payment authorization).

###### `Credential`

Credential object.

###### `Delegate`

Delegate object.

###### `DepositPreauth`

Deposit pre-authorization object.

###### `DID`

DID (Decentralized Identifier) object. Wire: `"did"`.

###### `Escrow`

Escrow object holding conditional or time-locked XRP.

###### `MPToken`

Multi-Purpose Token (MPT) holding. Wire: `"mptoken"`.

###### `MPTokenIssuance`

Multi-Purpose Token issuance object. Wire: `"mpt_issuance"`.

###### `NFTokenOffer`

NFToken buy or sell offer. Wire: `"nft_offer"`.

###### `NFTokenPage`

NFToken page storing up to 32 NFTokens. Wire: `"nft_page"`.

###### `Offer`

DEX limit order placed by the account.

###### `Oracle`

Oracle object.

###### `PayChannel`

Unidirectional XRP payment channel. Wire: `"payment_channel"`.

###### `PermissionedDomain`

Permissioned domain object.

###### `RippleState`

Trust line (RippleState) between two accounts. Wire: `"state"`.

###### `SignerList`

Multi-signature signer list attached to the account.

###### `Ticket`

Ticket reserving a future sequence number.

###### `XChainOwnedClaimId`

XChain owned claim ID. Wire: `"xchain_owned_claim_id"`.

###### `XChainOwnedCreateAccountClaimId`

XChain owned create account claim ID. Wire: `"xchain_owned_create_account_claim_id"`.

##### Implementations

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `AccountObjectsResponse`

Response payload for an [`AccountObjectsRequest`].

Contains the page of ledger objects owned by the queried account along with
ledger context and a pagination marker for retrieving subsequent pages.

# Examples

```rust
use xrpl::request::account_objects::AccountObjectsResponse;

fn object_count(resp: &AccountObjectsResponse) -> usize {
    resp.account_objects.len()
}
```

```rust
pub struct AccountObjectsResponse {
    pub account: String,
    pub account_objects: Vec<crate::types::AccountObject>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose objects are returned (r-address). |
| `account_objects` | `Vec<crate::types::AccountObject>` | Ledger objects owned by the account in the queried ledger. |
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger used to answer the request. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when querying the open ledger). |
| `limit` | `Option<u32>` | Effective page size applied by the server. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more objects remain on the next page. |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_offers`

Request and response types for the `account_offers` command.

```rust
pub mod account_offers { /* ... */ }
```

### Types

#### Struct `AccountOffersRequest`

Retrieves open DEX limit orders (offers) placed by an account.

Returns each standing offer's bid/ask amounts, quality, and optional expiration.
Paginate with `limit` and `marker` for accounts with many open orders.

# Examples

```rust
use xrpl::request::account_offers::AccountOffersRequest;

let req = AccountOffersRequest { limit: Some(100), ..AccountOffersRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") };
```

```rust
pub struct AccountOffersRequest {
    pub account: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose open DEX offers are queried (r-address). |
| `ledger_hash` | `Option<String>` | 64-hex-character hash of the ledger to query. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `limit` | `Option<u32>` | Maximum number of offers to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of offers to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountOffersResponse`

Response payload for an [`AccountOffersRequest`].

Contains the page of open DEX offers for the queried account along with ledger
context and a pagination marker for retrieving subsequent pages.

# Examples

```rust
use xrpl::request::account_offers::AccountOffersResponse;

fn total_offers(resp: &AccountOffersResponse) -> usize {
    resp.offers.len()
}
```

```rust
pub struct AccountOffersResponse {
    pub account: String,
    pub offers: Vec<AccountOffer>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
    pub limit: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose offers are returned (r-address). |
| `offers` | `Vec<AccountOffer>` | Open DEX limit orders placed by the account. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when querying the open ledger). |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger used to answer the request. |
| `ledger_hash` | `Option<String>` | Hash of the ledger used to answer the request. |
| `validated` | `Option<bool>` | `true` when the response is based on a validated (immutable) ledger. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more offers remain on the next page. |
| `limit` | `Option<u32>` | Effective page size applied by the server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountOffer`

A single open DEX limit order placed by an account.

Describes what the account is willing to exchange: `taker_gets` is what a taker
receives (what the account offers), and `taker_pays` is what the account demands
in return (what the taker must provide).

# Examples

```rust
use xrpl::request::account_offers::AccountOffer;
use xrpl::types::Amount;

fn is_sell_xrp(offer: &AccountOffer) -> bool {
    // taker_gets XRP means the account is selling XRP
    matches!(offer.taker_gets, Amount::Xrpl(_))
}
```

```rust
pub struct AccountOffer {
    pub flags: u32,
    pub seq: u32,
    pub taker_gets: crate::types::Amount,
    pub taker_pays: crate::types::Amount,
    pub quality: String,
    pub expiration: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `flags` | `u32` | Bitfield of offer flags (e.g. `lsfPassive`, `lsfSell`). |
| `seq` | `u32` | Sequence number of the `OfferCreate` transaction that placed this offer. |
| `taker_gets` | `crate::types::Amount` | Amount the taker receives when consuming this offer (what the account offers). |
| `taker_pays` | `crate::types::Amount` | Amount the taker must pay to consume this offer (what the account demands). |
| `quality` | `String` | Exchange rate (`taker_pays / taker_gets`) as a decimal string; lower is better for takers. |
| `expiration` | `Option<u32>` | Ripple epoch timestamp after which the offer expires and becomes unfillable. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `account_tx`

Request and response types for the `account_tx` command.

```rust
pub mod account_tx { /* ... */ }
```

### Types

#### Struct `AccountTxRequest`

Retrieves the transaction history for an account.

Returns all transactions that affected the account within the specified ledger
range. Use `ledger_index_min`/`ledger_index_max` to constrain the search window,
`forward` to control chronological order, and `limit`/`marker` to paginate.

# Examples

```rust
use xrpl::request::account_tx::AccountTxRequest;

let req = AccountTxRequest {
    limit: Some(50),
    forward: Some(true),
    ..AccountTxRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
};
```

```rust
pub struct AccountTxRequest {
    pub account: String,
    pub tx_type: Option<String>,
    pub ledger_index_min: Option<i64>,
    pub ledger_index_max: Option<i64>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub binary: Option<bool>,
    pub forward: Option<bool>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose transaction history is queried (r-address). |
| `tx_type` | `Option<String>` | Return only transactions of a specific type (e.g. `"AccountSet"`). *Clio server only.* |
| `ledger_index_min` | `Option<i64>` | Earliest ledger sequence to include; `-1` means the oldest available. |
| `ledger_index_max` | `Option<i64>` | Latest ledger sequence to include; `-1` means the most recent validated ledger. |
| `ledger_hash` | `Option<String>` | 64-hex-character hash identifying a single specific ledger to search. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger to query: a sequence number, or a shortcut such as `"validated"`. |
| `binary` | `Option<bool>` | When `true`, return transactions as raw hex instead of decoded JSON. |
| `forward` | `Option<bool>` | When `true`, return oldest transactions first (ascending order). |
| `limit` | `Option<u32>` | Maximum number of transactions to return in a single response. |
| `marker` | `Option<serde_json::Value>` | Pagination cursor returned by a previous response; pass back to fetch the next page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given account address.

- ```rust
  pub fn with_tx_type</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, tx_type: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the transaction type to filter by.

- ```rust
  pub fn with_ledger_index_min(self: Self, min: i64) -> Self { /* ... */ }
  ```
  Sets the minimum ledger index to search.

- ```rust
  pub fn with_ledger_index_max(self: Self, max: i64) -> Self { /* ... */ }
  ```
  Sets the maximum ledger index to search.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_binary(self: Self, binary: bool) -> Self { /* ... */ }
  ```
  Sets whether to return transactions in binary format.

- ```rust
  pub fn with_forward(self: Self, forward: bool) -> Self { /* ... */ }
  ```
  Sets the direction of traversal.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of transactions to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AccountTxResponse`

Response payload for an [`AccountTxRequest`].

Contains the page of transactions for the queried account along with the actual
ledger range searched and a pagination marker for retrieving subsequent pages.

# Examples

```rust
use xrpl::request::account_tx::AccountTxResponse;

fn has_more_pages(resp: &AccountTxResponse) -> bool {
    resp.marker.is_some()
}
```

```rust
pub struct AccountTxResponse {
    pub account: String,
    pub ledger_index_min: Option<i64>,
    pub ledger_index_max: Option<i64>,
    pub marker: Option<serde_json::Value>,
    pub transactions: Vec<AccountTransaction>,
    pub validated: Option<bool>,
    pub limit: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account whose transaction history is returned (r-address). |
| `ledger_index_min` | `Option<i64>` | Earliest ledger sequence actually searched (may differ from the requested value). |
| `ledger_index_max` | `Option<i64>` | Latest ledger sequence actually searched (may differ from the requested value). |
| `marker` | `Option<serde_json::Value>` | Pagination cursor; present when more transactions remain on the next page. |
| `transactions` | `Vec<AccountTransaction>` | Transactions affecting the account within the searched ledger range. |
| `validated` | `Option<bool>` | `true` when the response is based on validated (immutable) ledgers only. |
| `limit` | `Option<u32>` | Effective page size applied by the server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountTransaction`

A single transaction entry within an [`AccountTxResponse`].

Pairs the raw transaction JSON with its execution metadata. Always check
`validated` before using the data for financial decisions; only validated
transactions are final and irreversible.

# Examples

```rust
use xrpl::request::account_tx::AccountTransaction;
use xrpl::types::HasTransactionMeta;
fn print_delivered(tx: &AccountTransaction) {
    match tx.delivered_amount() {
        Some(amount) => println!("Delivered: {amount}"),
        None => println!("Not a payment transaction"),
    }
}
```

```rust
pub struct AccountTransaction {
    pub close_time_iso: Option<String>,
    pub hash: Option<String>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub meta: Option<crate::types::TransactionMeta>,
    pub meta_blob: Option<String>,
    pub tx_json: Option<serde_json::Value>,
    pub tx_blob: Option<String>,
    pub validated: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `close_time_iso` | `Option<String>` | The time the ledger containing this transaction was closed, in ISO 8601 format. |
| `hash` | `Option<String>` | The unique hash identifier of the transaction. |
| `ledger_hash` | `Option<String>` | A hex string of the ledger version that included this transaction. |
| `ledger_index` | `Option<u32>` | The ledger index of the ledger version that included this transaction. |
| `meta` | `Option<crate::types::TransactionMeta>` | Transaction execution metadata (JSON mode). |
| `meta_blob` | `Option<String>` | Transaction execution metadata as a hex string (Binary mode). |
| `tx_json` | `Option<serde_json::Value>` | Full transaction object as returned by the server (JSON mode). |
| `tx_blob` | `Option<String>` | A unique hex string defining the transaction (Binary mode). |
| `validated` | `bool` | `true` when the transaction is in a validated (immutable) ledger. |

##### Implementations

###### Methods

- ```rust
  pub fn flags(self: &Self) -> u32 { /* ... */ }
  ```
  Returns the raw transaction flags bitmask, or `0` if not present.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **HasTransactionMeta**
  - ```rust
    fn transaction_meta(self: &Self) -> Option<&TransactionMeta> { /* ... */ }
    ```

## Module `amm_info`

Request and response types for the `amm_info` command.

```rust
pub mod amm_info { /* ... */ }
```

### Types

#### Struct `AmmInfoRequest`

Retrieves the current state of an Automated Market Maker (AMM) pool.

Identify the pool either by its `amm_account` address or by the `asset`/`asset2` pair.

# Example
```rust
use xrpl::request::amm_info::AmmInfoRequest;
use xrpl::types::Asset;

let request = AmmInfoRequest {
    asset: Some(Asset::xrp()),
    asset2: Some(Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap()),
    ledger_index: Some("validated".into()),
    ..Default::default()
};
```

```rust
pub struct AmmInfoRequest {
    pub account: Option<String>,
    pub amm_account: Option<String>,
    pub asset: Option<crate::types::Asset>,
    pub asset2: Option<crate::types::Asset>,
    pub ledger_index: Option<serde_json::Value>,
    pub ledger_hash: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `Option<String>` | LP account to filter vote slots and auction slot by. |
| `amm_account` | `Option<String>` | AMM pool account address. |
| `asset` | `Option<crate::types::Asset>` | First asset in the pool pair (currency identifier only, no value). |
| `asset2` | `Option<crate::types::Asset>` | Second asset in the pool pair (currency identifier only, no value). |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |

##### Implementations

###### Methods

- ```rust
  pub fn by_account</* synthetic */ impl AsRef<str>: AsRef<str>>(amm_account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request to fetch an AMM by its specific account address.

- ```rust
  pub fn by_assets</* synthetic */ impl Into<Asset>: Into<Asset>, /* synthetic */ impl Into<Asset>: Into<Asset>>(asset: impl Into<Asset>, asset2: impl Into<Asset>) -> Self { /* ... */ }
  ```
  Creates a new request to fetch an AMM by its asset pair.

- ```rust
  pub fn with_account</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the LP account to filter vote slots and auction slot by.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the target ledger hash.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `AuthAccount`

An account authorized to trade at the discounted fee during the active auction slot.

```rust
pub struct AuthAccount {
    pub account: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Authorized account address. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AuctionSlot`

The active auction slot held by an LP, granting a discounted trading fee.

```rust
pub struct AuctionSlot {
    pub account: String,
    pub auth_accounts: Option<Vec<AuthAccount>>,
    pub discounted_fee: u32,
    pub expiration: String,
    pub price: crate::types::Amount,
    pub time_interval: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account holding the auction slot. |
| `auth_accounts` | `Option<Vec<AuthAccount>>` | Additional accounts authorized to trade at the discounted fee. |
| `discounted_fee` | `u32` | Trading fee the slot holder pays, in units of 1/100,000. |
| `expiration` | `String` | ISO 8601 expiration time of the slot. |
| `price` | `crate::types::Amount` | LP token amount paid for the slot. |
| `time_interval` | `u32` | Current 72-minute time interval within the 24-hour auction window. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `VoteSlot`

An LP's vote on the pool trading fee.

```rust
pub struct VoteSlot {
    pub account: String,
    pub trading_fee: u32,
    pub vote_weight: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account that cast the vote. |
| `trading_fee` | `u32` | Proposed trading fee in units of 1/100,000. |
| `vote_weight` | `u32` | Weight of this vote, proportional to the LP's token share. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AmmDescription`

Full description of an AMM pool returned by `amm_info`.

```rust
pub struct AmmDescription {
    pub account: String,
    pub amount: crate::types::Amount,
    pub amount2: crate::types::Amount,
    pub asset_frozen: Option<bool>,
    pub asset2_frozen: Option<bool>,
    pub auction_slot: Option<AuctionSlot>,
    pub lp_token: crate::types::Amount,
    pub trading_fee: u32,
    pub vote_slots: Option<Vec<VoteSlot>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | AMM pool account address on the ledger. |
| `amount` | `crate::types::Amount` | Balance of the first asset held by the pool. |
| `amount2` | `crate::types::Amount` | Balance of the second asset held by the pool. |
| `asset_frozen` | `Option<bool>` | Whether the first asset is currently frozen by its issuer. |
| `asset2_frozen` | `Option<bool>` | Whether the second asset is currently frozen by its issuer. |
| `auction_slot` | `Option<AuctionSlot>` | Active auction slot, if one has been purchased. |
| `lp_token` | `crate::types::Amount` | Outstanding LP token supply for this pool. |
| `trading_fee` | `u32` | Current trading fee in units of 1/100,000 (e.g. 500 = 0.5%). |
| `vote_slots` | `Option<Vec<VoteSlot>>` | LP fee votes currently in effect. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AmmInfoResponse`

Response to an `amm_info` request.

```rust
pub struct AmmInfoResponse {
    pub amm: AmmDescription,
    pub ledger_current_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amm` | `AmmDescription` | AMM pool state. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `book_offers`

Request and response types for the `book_offers` command.

```rust
pub mod book_offers { /* ... */ }
```

### Types

#### Struct `BookOffersRequest`

Retrieves a list of offers between two assets from the order book.

# Examples

Using the constructor for the common case:
```rust
use xrpl::request::book_offers::BookOffersRequest;
use xrpl::types::Asset;

let request = BookOffersRequest::new(
    Asset::xrp(),
    Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
)
.with_limit(20)
.with_ledger_index("validated");
```

Using struct literal syntax when all fields must be explicit:
```rust
use xrpl::request::book_offers::BookOffersRequest;
use xrpl::types::Asset;

let request = BookOffersRequest {
    taker_gets: Asset::xrp(),
    taker_pays: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    limit: Some(20),
    ledger_index: Some("validated".into()),
    taker: None,
    ledger_hash: None,
    domain: None,
};
```

```rust
pub struct BookOffersRequest {
    pub taker_gets: crate::types::Asset,
    pub taker_pays: crate::types::Asset,
    pub domain: Option<String>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub taker: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `taker_gets` | `crate::types::Asset` | Asset the taker receives (defines one side of the order book). |
| `taker_pays` | `crate::types::Asset` | Asset the taker pays (defines the other side of the order book). |
| `domain` | `Option<String>` | If provided, return offers from the corresponding permissioned DEX. |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `limit` | `Option<u32>` | Maximum number of offers to return. |
| `taker` | `Option<String>` | Account to use as perspective for unfunded offers. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl Into<Asset>: Into<Asset>, /* synthetic */ impl Into<Asset>: Into<Asset>>(taker_gets: impl Into<Asset>, taker_pays: impl Into<Asset>) -> Self { /* ... */ }
  ```
  Creates a new request for the order book between two assets.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of offers to return.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query ("validated", "closed", "current", or a number).

- ```rust
  pub fn with_taker</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, taker: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the account whose perspective is used for computing unfunded offer amounts.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Targets a specific ledger version by its 64-character hex hash.

- ```rust
  pub fn with_domain</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, domain: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the optional `domain` field for permissioned DEXs.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `BookOffersResponse`

Response to a `book_offers` request.

```rust
pub struct BookOffersResponse {
    pub offers: Vec<BookOffer>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `offers` | `Vec<BookOffer>` | Ordered list of offers, best quality first. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `BookOffer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A single offer entry from an order book, including rippled-computed quality fields.

```rust
pub struct BookOffer {
    pub account: String,
    pub flags: u32,
    pub sequence: u32,
    pub taker_gets: crate::types::Amount,
    pub taker_pays: crate::types::Amount,
    pub book_directory: Option<String>,
    pub book_node: Option<String>,
    pub expiration: Option<u32>,
    pub quality: Option<String>,
    pub owner_funds: Option<String>,
    pub taker_gets_funded: Option<crate::types::Amount>,
    pub taker_pays_funded: Option<crate::types::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account that placed the offer. |
| `flags` | `u32` | Offer flags bit field. |
| `sequence` | `u32` | Sequence number that identifies the offer on the ledger. |
| `taker_gets` | `crate::types::Amount` | Amount the offer creator receives when the offer executes. |
| `taker_pays` | `crate::types::Amount` | Amount the offer creator pays when the offer executes. |
| `book_directory` | `Option<String>` | Index of the book directory page containing this offer. |
| `book_node` | `Option<String>` | Position of this offer within its book directory page. |
| `expiration` | `Option<u32>` | Ripple epoch timestamp after which the offer expires. |
| `quality` | `Option<String>` | Exchange rate (taker_pays / taker_gets), higher is better for the taker. |
| `owner_funds` | `Option<String>` | The account's available balance of `taker_gets`. Omitted for XRP. |
| `taker_gets_funded` | `Option<crate::types::Amount>` | Adjusted `taker_gets` after considering `owner_funds`. |
| `taker_pays_funded` | `Option<crate::types::Amount>` | Adjusted `taker_pays` after considering `owner_funds`. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `fee`

Request and response types for the `fee` command.

```rust
pub mod fee { /* ... */ }
```

### Types

#### Struct `FeeRequest`

Retrieves the current transaction cost levels from the server.

Useful for choosing an appropriate fee before submitting a transaction.

# Example
```rust
use xrpl::request::fee::FeeRequest;

let request = FeeRequest;
```

```rust
pub struct FeeRequest;
```

##### Implementations

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `FeeResult`

Response to a `fee` request.

```rust
pub struct FeeResult {
    pub current_ledger_size: String,
    pub current_queue_size: String,
    pub drops: FeeDrops,
    pub expected_ledger_size: String,
    pub ledger_current_index: u32,
    pub levels: FeeLevels,
    pub max_queue_size: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `current_ledger_size` | `String` | Number of transactions provisionally included in the in-progress ledger. |
| `current_queue_size` | `String` | Number of transactions currently queued for the next ledger. |
| `drops` | `FeeDrops` | Fee levels expressed in drops of XRP. |
| `expected_ledger_size` | `String` | The approximate number of transactions expected to be included in the current ledger. |
| `ledger_current_index` | `u32` | Sequence number of the current open ledger. |
| `levels` | `FeeLevels` | Fee levels expressed in abstract fee units (useful for relative comparisons). |
| `max_queue_size` | `String` | The maximum number of transactions that the transaction queue can currently hold. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `FeeDrops`

Transaction cost thresholds in drops of XRP.

```rust
pub struct FeeDrops {
    pub base_fee: String,
    pub median_fee: String,
    pub minimum_fee: String,
    pub open_ledger_fee: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `base_fee` | `String` | Cost of a reference transaction at normal load. |
| `median_fee` | `String` | Median fee among recently validated transactions. |
| `minimum_fee` | `String` | Minimum fee that will be accepted by the node into its queue. |
| `open_ledger_fee` | `String` | Minimum fee to be included in the current open ledger immediately. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `FeeLevels`

Transaction cost thresholds expressed in abstract fee units (1 unit = base_fee / 256).

```rust
pub struct FeeLevels {
    pub median_level: String,
    pub minimum_level: String,
    pub open_ledger_level: String,
    pub reference_level: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `median_level` | `String` | Median fee level among recently validated transactions. |
| `minimum_level` | `String` | Minimum fee level accepted into the node's queue. |
| `open_ledger_level` | `String` | Minimum fee level to enter the current open ledger immediately. |
| `reference_level` | `String` | The equivalent of the minimum transaction cost, represented in fee levels. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger`

Request and response types for the `ledger` command.

```rust
pub mod ledger { /* ... */ }
```

### Types

#### Struct `LedgerRequest`

Retrieves information about a specific ledger version.

# Example
```rust
use xrpl::request::ledger::LedgerRequest;

let request = LedgerRequest {
    ledger_index: Some("validated".into()),
    transactions: Some(true),
    ..Default::default()
};
```

```rust
pub struct LedgerRequest {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub full: Option<bool>,
    pub accounts: Option<bool>,
    pub transactions: Option<bool>,
    pub expand: Option<bool>,
    pub owner_funds: Option<bool>,
    pub binary: Option<bool>,
    pub queue: Option<bool>,
    pub diff: Option<bool>,
    pub entry_type: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `full` | `Option<bool>` | Return full information on all accounts in the ledger (very large). |
| `accounts` | `Option<bool>` | Return information on all accounts in the ledger (very large). |
| `transactions` | `Option<bool>` | Return information on all transactions. |
| `expand` | `Option<bool>` | Return full details of transactions and accounts rather than hashes. |
| `owner_funds` | `Option<bool>` | Include `owner_funds` field on offer transactions. |
| `binary` | `Option<bool>` | Return transaction information in binary format. |
| `queue` | `Option<bool>` | Include queued transactions in the results. |
| `diff` | `Option<bool>` | (Clio only) Return array of hashes that were added, modified, or deleted. |
| `entry_type` | `Option<String>` | (Admin only) Filter results by ledger entry type. |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_ledger_hash(self: Self, ledger_hash: &str) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_ledger_index<T: Into<Value>>(self: Self, ledger_index: T) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_full(self: Self, full: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_accounts(self: Self, accounts: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_transactions(self: Self, transactions: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_expand(self: Self, expand: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_owner_funds(self: Self, owner_funds: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_binary(self: Self, binary: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_queue(self: Self, queue: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_diff(self: Self, diff: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_entry_type(self: Self, entry_type: &str) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `LedgerResponse`

Response to a `ledger` request.

```rust
pub struct LedgerResponse {
    pub ledger: LedgerInfo,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
    pub queue_data: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger` | `LedgerInfo` | Ledger header and optional transaction/account data. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version returned. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version returned. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |
| `queue_data` | `Option<serde_json::Value>` | Queued transactions affecting this ledger (present when `queue` is `true`). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `LedgerInfo`

Ledger header fields returned inside a `LedgerResponse`.

```rust
pub struct LedgerInfo {
    pub account_hash: Option<String>,
    pub close_flags: Option<u32>,
    pub close_time: Option<u64>,
    pub close_time_human: Option<String>,
    pub close_time_resolution: Option<u32>,
    pub close_time_iso: Option<String>,
    pub closed: bool,
    pub ledger_hash: String,
    pub ledger_index: u32,
    pub parent_close_time: Option<u64>,
    pub parent_hash: Option<String>,
    pub total_coins: String,
    pub transaction_hash: String,
    pub transactions: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account_hash` | `Option<String>` | Root hash of the account state tree. |
| `close_flags` | `Option<u32>` | A bit-map of flags relating to the closing of this ledger. |
| `close_time` | `Option<u64>` | Close time as Ripple epoch seconds. |
| `close_time_human` | `Option<String>` | Close time in human-readable UTC format. |
| `close_time_resolution` | `Option<u32>` | Rounding applied to the close time, in seconds. |
| `close_time_iso` | `Option<String>` | Close time in ISO 8601 format. |
| `closed` | `bool` | Whether the ledger has been closed. |
| `ledger_hash` | `String` | Unique identifying hash of this ledger version. |
| `ledger_index` | `u32` | Sequence number of this ledger. |
| `parent_close_time` | `Option<u64>` | Close time of the parent ledger as Ripple epoch seconds. |
| `parent_hash` | `Option<String>` | Hash of the immediately preceding ledger. |
| `total_coins` | `String` | Total XRP in existence, in drops. |
| `transaction_hash` | `String` | Root hash of the transaction tree. |
| `transactions` | `Option<serde_json::Value>` | Transaction hashes or expanded transaction objects (depending on `expand`). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger_closed`

Request and response types for the `ledger_closed` command.

```rust
pub mod ledger_closed { /* ... */ }
```

### Types

#### Struct `LedgerClosedRequest`

Returns the unique identifiers of the most recently closed ledger.

```rust
pub struct LedgerClosedRequest;
```

##### Implementations

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `LedgerClosedResponse`

Response to a `ledger_closed` request.

```rust
pub struct LedgerClosedResponse {
    pub ledger_hash: String,
    pub ledger_index: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_hash` | `String` | Hash of the most recently closed ledger. |
| `ledger_index` | `u32` | Sequence number of the most recently closed ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger_current`

Request and response types for the `ledger_current` command.

```rust
pub mod ledger_current { /* ... */ }
```

### Types

#### Struct `LedgerCurrentRequest`

Returns the sequence number of the current open ledger.

```rust
pub struct LedgerCurrentRequest;
```

##### Implementations

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `LedgerCurrentResponse`

Response to a `ledger_current` request.

```rust
pub struct LedgerCurrentResponse {
    pub ledger_current_index: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_current_index` | `u32` | Sequence number of the current open ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger_data`

Request and response types for the `ledger_data` command.

```rust
pub mod ledger_data { /* ... */ }
```

### Types

#### Struct `LedgerDataRequest`

Returns all ledger objects in a given ledger version, paginated by marker.

Used for scanning the entire ledger state. For looking up specific entries
prefer `ledger_entry`.

```rust
pub struct LedgerDataRequest {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub binary: Option<bool>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
    pub entry_type: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `binary` | `Option<bool>` | If true, return entries as binary blobs instead of JSON. |
| `limit` | `Option<u32>` | Maximum number of entries per page. |
| `marker` | `Option<serde_json::Value>` | Opaque pagination cursor from a previous response; omit for the first page. |
| `entry_type` | `Option<String>` | Filter results to a specific type of ledger entry. |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_ledger_hash(self: Self, ledger_hash: &str) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_ledger_index<T: Into<Value>>(self: Self, ledger_index: T) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_binary(self: Self, binary: bool) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_marker<T: Into<Value>>(self: Self, marker: T) -> Self { /* ... */ }
  ```

- ```rust
  pub fn with_entry_type(self: Self, entry_type: &str) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `LedgerDataResponse`

Response to a `ledger_data` request.

```rust
pub struct LedgerDataResponse {
    pub ledger: Option<serde_json::Value>,
    pub ledger_hash: String,
    pub ledger_index: u32,
    pub marker: Option<serde_json::Value>,
    pub state: Vec<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger` | `Option<serde_json::Value>` | The complete ledger header data for this ledger version. |
| `ledger_hash` | `String` | Hash of the ledger version scanned. |
| `ledger_index` | `u32` | Sequence number of the ledger version scanned. |
| `marker` | `Option<serde_json::Value>` | Marker for the next page. Absent when the last page has been returned. |
| `state` | `Vec<serde_json::Value>` | Ledger entry objects. Each entry includes an `index` field with the entry hash. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger_entry`

Request and response types for the `ledger_entry` command.

```rust
pub mod ledger_entry { /* ... */ }
```

### Types

#### Struct `LedgerEntryRequest`

Retrieves a single ledger entry by its identifying key.

Set exactly one of the key fields. All other key fields must be `None`.

# Examples

Look up a ledger entry directly by its index:
```rust
use xrpl::request::ledger_entry::LedgerEntryRequest;
let request = LedgerEntryRequest::by_index("7DB0788C020F02780A673DC74757F23823FA3014C1866E72CC4CD8B226CD6EF4")
    .with_ledger_index("validated");
```

Look up an account's root object:
```rust
use xrpl::request::ledger_entry::LedgerEntryRequest;
let request = LedgerEntryRequest::for_account_root("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
```

```rust
pub struct LedgerEntryRequest {
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub binary: Option<bool>,
    pub include_deleted: Option<bool>,
    pub index: Option<String>,
    pub account_root: Option<String>,
    pub amendments: Option<String>,
    pub check: Option<String>,
    pub fee: Option<String>,
    pub hashes: Option<String>,
    pub nunl: Option<String>,
    pub nft_page: Option<String>,
    pub nft_offer: Option<String>,
    pub payment_channel: Option<String>,
    pub did: Option<String>,
    pub mpt_issuance: Option<String>,
    pub signer_list: Option<String>,
    pub vault: Option<String>,
    pub escrow: Option<EscrowLedgerKey>,
    pub offer: Option<OfferLedgerKey>,
    pub ripple_state: Option<RippleStateLedgerKey>,
    pub ticket: Option<TicketLedgerKey>,
    pub deposit_preauth: Option<DepositPreauthLedgerKey>,
    pub amm: Option<AmmLedgerKey>,
    pub directory: Option<serde_json::Value>,
    pub bridge: Option<serde_json::Value>,
    pub oracle: Option<serde_json::Value>,
    pub credential: Option<serde_json::Value>,
    pub xchain_owned_claim_id: Option<serde_json::Value>,
    pub xchain_owned_create_account_claim_id: Option<serde_json::Value>,
    pub loan: Option<serde_json::Value>,
    pub loan_broker: Option<serde_json::Value>,
    pub mptoken: Option<serde_json::Value>,
    pub permissioned_domain: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `binary` | `Option<bool>` | If true, return the entry as a binary blob instead of JSON. |
| `include_deleted` | `Option<bool>` | (Clio only) Return the complete data as it was prior to its deletion if the queried object has been deleted. |
| `index` | `Option<String>` | Direct lookup by the 64-character hex ledger entry index. |
| `account_root` | `Option<String>` | Account address for an AccountRoot entry. |
| `amendments` | `Option<String>` | The Amendments entry. |
| `check` | `Option<String>` | Check ID (64-character hex). |
| `fee` | `Option<String>` | The FeeSettings entry. |
| `hashes` | `Option<String>` | The LedgerHashes entry. |
| `nunl` | `Option<String>` | The NegativeUNL entry. |
| `nft_page` | `Option<String>` | The NFT Page ID. |
| `nft_offer` | `Option<String>` | NFToken offer ID. |
| `payment_channel` | `Option<String>` | Payment channel ID. |
| `did` | `Option<String>` | Account address for a DID entry. |
| `mpt_issuance` | `Option<String>` | The MPTokenIssuance ID. |
| `signer_list` | `Option<String>` | The SignerList ID. |
| `vault` | `Option<String>` | The Vault ID. |
| `escrow` | `Option<EscrowLedgerKey>` | Key for an Escrow entry. |
| `offer` | `Option<OfferLedgerKey>` | Key for an Offer entry. |
| `ripple_state` | `Option<RippleStateLedgerKey>` | Key for a trust line (RippleState) entry. |
| `ticket` | `Option<TicketLedgerKey>` | Key for a Ticket entry. |
| `deposit_preauth` | `Option<DepositPreauthLedgerKey>` | Key for a DepositPreauth entry. |
| `amm` | `Option<AmmLedgerKey>` | Key for an AMM pool entry. |
| `directory` | `Option<serde_json::Value>` | Key for a DirectoryNode entry. |
| `bridge` | `Option<serde_json::Value>` | Key for an XChainBridge entry. |
| `oracle` | `Option<serde_json::Value>` | Key for a PriceOracle entry. |
| `credential` | `Option<serde_json::Value>` | Key for a Credential entry. |
| `xchain_owned_claim_id` | `Option<serde_json::Value>` | Key for an XChainOwnedClaimID entry. |
| `xchain_owned_create_account_claim_id` | `Option<serde_json::Value>` | Key for an XChainOwnedCreateAccountClaimID entry. |
| `loan` | `Option<serde_json::Value>` | Key for a Loan entry. |
| `loan_broker` | `Option<serde_json::Value>` | Key for a LoanBroker entry. |
| `mptoken` | `Option<serde_json::Value>` | Key for an MPToken entry. |
| `permissioned_domain` | `Option<serde_json::Value>` | Key for a PermissionedDomain entry. |

##### Implementations

###### Methods

- ```rust
  pub fn by_index</* synthetic */ impl AsRef<str>: AsRef<str>>(index: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request to look up an entry by its 64-character hex index.

- ```rust
  pub fn for_account_root</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for an AccountRoot entry.

- ```rust
  pub fn for_escrow</* synthetic */ impl AsRef<str>: AsRef<str>>(owner: impl AsRef<str>, seq: u32) -> Self { /* ... */ }
  ```
  Creates a request for an Escrow entry identified by owner and sequence number.

- ```rust
  pub fn for_offer</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, seq: u32) -> Self { /* ... */ }
  ```
  Creates a request for an Offer entry identified by account and sequence number.

- ```rust
  pub fn for_ripple_state</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(accounts: [impl AsRef<str>; 2], currency: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a trust line (RippleState) entry identified by two accounts and a currency code.

- ```rust
  pub fn for_amm</* synthetic */ impl Into<Asset>: Into<Asset>, /* synthetic */ impl Into<Asset>: Into<Asset>>(asset: impl Into<Asset>, asset2: impl Into<Asset>) -> Self { /* ... */ }
  ```
  Creates a request for an AMM pool entry identified by its two assets.

- ```rust
  pub fn for_check</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a Check entry.

- ```rust
  pub fn for_nft_page</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for an NFTokenPage entry.

- ```rust
  pub fn for_nft_offer</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for an NFTokenOffer entry.

- ```rust
  pub fn for_payment_channel</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a PaymentChannel entry.

- ```rust
  pub fn for_did</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a DID entry.

- ```rust
  pub fn for_mpt_issuance</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for an MPTokenIssuance entry.

- ```rust
  pub fn for_signer_list</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a SignerList entry.

- ```rust
  pub fn for_vault</* synthetic */ impl AsRef<str>: AsRef<str>>(id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a Vault entry.

- ```rust
  pub fn for_ticket</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, ticket_seq: u32) -> Self { /* ... */ }
  ```
  Creates a request for a Ticket entry identified by account and ticket sequence number.

- ```rust
  pub fn for_deposit_preauth</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(owner: impl AsRef<str>, authorized: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request for a DepositPreauth entry identified by owner and authorized account.

- ```rust
  pub fn for_amendments() -> Self { /* ... */ }
  ```
  Creates a request for the Amendments singleton entry.

- ```rust
  pub fn for_fee_settings() -> Self { /* ... */ }
  ```
  Creates a request for the FeeSettings singleton entry.

- ```rust
  pub fn for_hashes() -> Self { /* ... */ }
  ```
  Creates a request for the LedgerHashes singleton entry.

- ```rust
  pub fn for_nunl() -> Self { /* ... */ }
  ```
  Creates a request for the NegativeUNL singleton entry.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the target ledger hash.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut.

- ```rust
  pub fn with_binary(self: Self, binary: bool) -> Self { /* ... */ }
  ```
  Configures whether to return binary data instead of JSON.

- ```rust
  pub fn with_include_deleted(self: Self, include_deleted: bool) -> Self { /* ... */ }
  ```
  Configures whether to include deleted objects.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `LedgerEntryResponse`

Response to a `ledger_entry` request.

```rust
pub struct LedgerEntryResponse {
    pub index: String,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub node: Option<serde_json::Value>,
    pub node_binary: Option<String>,
    pub validated: Option<bool>,
    pub deleted_ledger_index: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `index` | `String` | The 64-character hex index of the ledger entry. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `node` | `Option<serde_json::Value>` | The ledger entry in JSON format. `None` when `binary` is `true`. |
| `node_binary` | `Option<String>` | The ledger entry in binary format. `None` when `binary` is `false`. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |
| `deleted_ledger_index` | `Option<String>` | (Clio server only) The ledger index where the ledger entry object was deleted. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `EscrowLedgerKey`

Key for looking up an Escrow entry: `{owner, seq}`.

```rust
pub struct EscrowLedgerKey {
    pub owner: String,
    pub seq: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `owner` | `String` | Account that created the escrow. |
| `seq` | `u32` | Sequence number of the EscrowCreate transaction. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(owner: impl AsRef<str>, seq: u32) -> Self { /* ... */ }
  ```
  Creates a new key for Escrow lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `OfferLedgerKey`

Key for looking up an Offer entry: `{account, seq}`.

```rust
pub struct OfferLedgerKey {
    pub account: String,
    pub seq: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account that placed the offer. |
| `seq` | `u32` | Sequence number of the OfferCreate transaction. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, seq: u32) -> Self { /* ... */ }
  ```
  Creates a new key for Offer lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `RippleStateLedgerKey`

Key for looking up a trust line (RippleState): `{accounts: [A, B], currency}`.

```rust
pub struct RippleStateLedgerKey {
    pub accounts: [String; 2],
    pub currency: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `accounts` | `[String; 2]` | The two accounts sharing this trust line (order does not matter). |
| `currency` | `String` | ISO 4217 currency code or 40-character hex non-standard currency code. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(account1: impl AsRef<str>, account2: impl AsRef<str>, currency: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new key for RippleState lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `TicketLedgerKey`

Key for looking up a Ticket entry: `{account, ticket_seq}`.

```rust
pub struct TicketLedgerKey {
    pub account: String,
    pub ticket_seq: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Account that created the ticket. |
| `ticket_seq` | `u32` | Sequence number reserved by the ticket. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, ticket_seq: u32) -> Self { /* ... */ }
  ```
  Creates a new key for Ticket lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `DepositPreauthLedgerKey`

Key for looking up a DepositPreauth entry: `{owner, authorized}`.

```rust
pub struct DepositPreauthLedgerKey {
    pub owner: String,
    pub authorized: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `owner` | `String` | Account that granted the preauthorization. |
| `authorized` | `String` | Account that was preauthorized to send payments. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(owner: impl AsRef<str>, authorized: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new key for DepositPreauth lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `AmmLedgerKey`

Key for looking up an AMM entry by its two assets.

```rust
pub struct AmmLedgerKey {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset in the AMM pair. |
| `asset2` | `crate::types::Asset` | Second asset in the AMM pair. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl Into<Asset>: Into<Asset>, /* synthetic */ impl Into<Asset>: Into<Asset>>(asset: impl Into<Asset>, asset2: impl Into<Asset>) -> Self { /* ... */ }
  ```
  Creates a new key for AMM lookup.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `nft_buy_offers`

Request and response types for the `nft_buy_offers` command.

```rust
pub mod nft_buy_offers { /* ... */ }
```

### Types

#### Struct `NftBuyOffersRequest`

Retrieves all buy offers for a specific NFToken.

```rust
pub struct NftBuyOffersRequest {
    pub nft_id: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nft_id` | `String` | 64-character hex NFToken ID to query buy offers for. |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `limit` | `Option<u32>` | Maximum number of offers per page. |
| `marker` | `Option<serde_json::Value>` | Opaque pagination cursor from a previous response; omit for the first page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(nft_id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given NFToken ID.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of offers to return per page.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query ("validated", "closed", "current", or a number).

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Targets a specific ledger version by its 64-character hex hash.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the opaque pagination cursor from a previous response.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `NftBuyOffersResponse`

Response to an `nft_buy_offers` request.

```rust
pub struct NftBuyOffersResponse {
    pub nft_id: String,
    pub offers: Vec<NftOffer>,
    pub limit: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nft_id` | `String` | NFToken ID the offers are for. |
| `offers` | `Vec<NftOffer>` | Buy offers for the NFToken. |
| `limit` | `Option<u32>` | Limit the number of NFT buy offers to retrieve. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |
| `marker` | `Option<serde_json::Value>` | Opaque pagination cursor; present when more pages are available. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `NftOffer`

A single NFToken buy offer returned by `nft_buy_offers`.

```rust
pub struct NftOffer {
    pub amount: crate::types::Amount,
    pub flags: u32,
    pub nft_offer_index: String,
    pub owner: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Offered amount for the NFToken. |
| `flags` | `u32` | Offer flags bit field. |
| `nft_offer_index` | `String` | Ledger index (ID) of the NFTokenOffer object. |
| `owner` | `String` | Account that placed the offer. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `nft_sell_offers`

Request and response types for the `nft_sell_offers` command.

```rust
pub mod nft_sell_offers { /* ... */ }
```

### Types

#### Struct `NftSellOffersRequest`

Retrieves all sell offers for a specific NFToken.

```rust
pub struct NftSellOffersRequest {
    pub nft_id: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
    pub limit: Option<u32>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nft_id` | `String` | 64-character hex NFToken ID to query sell offers for. |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |
| `limit` | `Option<u32>` | Maximum number of offers per page. |
| `marker` | `Option<serde_json::Value>` | Opaque pagination cursor from a previous response; omit for the first page. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(nft_id: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given NFToken ID.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the ledger hash to query.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut to query.

- ```rust
  pub fn with_limit(self: Self, limit: u32) -> Self { /* ... */ }
  ```
  Sets the maximum number of offers to return.

- ```rust
  pub fn with_marker</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, marker: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the pagination marker.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `NftSellOffersResponse`

Response to an `nft_sell_offers` request.

```rust
pub struct NftSellOffersResponse {
    pub nft_id: String,
    pub offers: Vec<NftOffer>,
    pub limit: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub ledger_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub validated: Option<bool>,
    pub marker: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nft_id` | `String` | NFToken ID the offers are for. |
| `offers` | `Vec<NftOffer>` | Sell offers for the NFToken. |
| `limit` | `Option<u32>` | Limit the number of NFT sell offers to retrieve. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |
| `marker` | `Option<serde_json::Value>` | Opaque pagination cursor; present when more pages are available. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `NftOffer`

An NFToken offer returned by `nft_buy_offers` and `nft_sell_offers`.

```rust
pub struct NftOffer {
    pub amount: crate::types::Amount,
    pub flags: u32,
    pub nft_offer_index: String,
    pub owner: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Offered or asking amount for the NFToken. |
| `flags` | `u32` | Offer flags bit field. |
| `nft_offer_index` | `String` | Ledger index (ID) of the NFTokenOffer object. |
| `owner` | `String` | Account that placed the offer. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ripple_path_find`

Request and response types for the `ripple_path_find` command.

```rust
pub mod ripple_path_find { /* ... */ }
```

### Types

#### Struct `RipplePathFindRequest`

Finds a payment path between a source and destination account (single-shot).

Returns a list of path alternatives sorted by quality. Use the best
alternative's `source_amount` when building the `Payment` transaction.

# Example
```rust
use xrpl::request::ripple_path_find::RipplePathFindRequest;
use xrpl::types::Amount;

let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
let request = RipplePathFindRequest::new("rSource...", "rDest...", amount);
```

```rust
pub struct RipplePathFindRequest {
    pub source_account: String,
    pub destination_account: String,
    pub destination_amount: crate::types::Amount,
    pub domain: Option<String>,
    pub send_max: Option<crate::types::Amount>,
    pub source_currencies: Option<Vec<crate::types::Asset>>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `source_account` | `String` | Account that will send the payment. |
| `destination_account` | `String` | Account that will receive the payment. |
| `destination_amount` | `crate::types::Amount` | Amount the destination account should receive. |
| `domain` | `Option<String>` | If provided, only return paths that use the corresponding permissioned DEX. |
| `send_max` | `Option<crate::types::Amount>` | Maximum amount the source account is willing to spend. |
| `source_currencies` | `Option<Vec<crate::types::Asset>>` | Currencies the source account may use. Defaults to all available. |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl Into<Amount>: Into<Amount>>(source_account: impl AsRef<str>, destination_account: impl AsRef<str>, destination_amount: impl Into<Amount>) -> Self { /* ... */ }
  ```
  Creates a new request with the mandatory source, destination, and amount fields.

- ```rust
  pub fn with_domain</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, domain: impl AsRef<str>) -> Self { /* ... */ }
  ```
  If provided, only return paths that use the corresponding permissioned DEX.

- ```rust
  pub fn with_send_max</* synthetic */ impl Into<Amount>: Into<Amount>>(self: Self, send_max: impl Into<Amount>) -> Self { /* ... */ }
  ```
  Maximum amount the source account is willing to spend.

- ```rust
  pub fn with_source_currencies(self: Self, source_currencies: Vec<Asset>) -> Self { /* ... */ }
  ```
  Currencies the source account may use. Defaults to all available.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Ledger hash to target a specific ledger version.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Ledger index or shortcut ("validated", "closed", "current").

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `RipplePathFindResponse`

Response to a `ripple_path_find` request.

```rust
pub struct RipplePathFindResponse {
    pub alternatives: Vec<PathAlternative>,
    pub destination_account: String,
    pub destination_amount: crate::types::Amount,
    pub destination_currencies: Option<Vec<String>>,
    pub source_account: String,
    pub full_reply: Option<bool>,
    pub ledger_current_index: Option<u32>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub validated: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `alternatives` | `Vec<PathAlternative>` | Available path alternatives, sorted by quality (best first). |
| `destination_account` | `String` | Destination account from the request. |
| `destination_amount` | `crate::types::Amount` | Destination amount from the request. |
| `destination_currencies` | `Option<Vec<String>>` | Currencies the destination account accepts. |
| `source_account` | `String` | Source account from the request. |
| `full_reply` | `Option<bool>` | Whether the response is complete (not a partial streaming update). |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (unvalidated results). |
| `ledger_hash` | `Option<String>` | Hash of the ledger version used. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version used. |
| `validated` | `Option<bool>` | Whether the data comes from a validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `PathAlternative`

A single payment path alternative returned by `ripple_path_find`.

```rust
pub struct PathAlternative {
    pub paths_computed: serde_json::Value,
    pub source_amount: crate::types::Amount,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `paths_computed` | `serde_json::Value` | Computed payment paths in XRPL path format. |
| `source_amount` | `crate::types::Amount` | Amount the source account must send along this path. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `server_info`

Request and response types for the `server_info` command.

```rust
pub mod server_info { /* ... */ }
```

### Types

#### Struct `ServerInfoRequest`

Retrieves a human-readable summary of the server's state and ledger chain.

Useful for health checks, build version detection, and monitoring sync status.

# Example
```rust
use xrpl::request::server_info::ServerInfoRequest;

let request = ServerInfoRequest::new().with_counters(true);
```

```rust
pub struct ServerInfoRequest {
    pub counters: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `counters` | `Option<bool>` | If `true`, return metrics about the job queue, ledger store, and API method activity. |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Creates a new request with default options.

- ```rust
  pub fn with_counters(self: Self, counters: bool) -> Self { /* ... */ }
  ```
  Requests job queue, ledger store, and API method activity metrics.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `ServerInfoResult`

Response to a `server_info` request.

```rust
pub struct ServerInfoResult {
    pub info: ServerInfo,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `info` | `ServerInfo` | Detailed server state information. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfo`

Server state details returned inside a `ServerInfoResult`.

```rust
pub struct ServerInfo {
    pub amendment_blocked: Option<bool>,
    pub build_version: String,
    pub clio_server_url: Option<String>,
    pub closed_ledger: Option<serde_json::Value>,
    pub complete_ledgers: String,
    pub hostid: String,
    pub io_latency_ms: u64,
    pub jq_trans_overflow: String,
    pub last_close: ServerInfoLastClose,
    pub load_factor: f64,
    pub load_factor_fee_escalation: Option<f64>,
    pub load_factor_fee_queue: Option<f64>,
    pub load_factor_fee_reference: Option<f64>,
    pub load_factor_server: Option<f64>,
    pub network_id: Option<u64>,
    pub peer_disconnects: String,
    pub peer_disconnects_resources: String,
    pub peers: u64,
    pub ports: Option<Vec<ServerInfoPort>>,
    pub pubkey_node: String,
    pub reporting: Option<ServerInfoReporting>,
    pub server_state: String,
    pub server_state_duration_us: String,
    pub state_accounting: ServerInfoStateAccounting,
    pub time: String,
    pub uptime: u64,
    pub validated_ledger: Option<ServerInfoValidatedLedger>,
    pub validation_quorum: u64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amendment_blocked` | `Option<bool>` | Whether the server is blocked from participating due to unsupported amendments. |
| `build_version` | `String` | rippled build version string. |
| `clio_server_url` | `Option<String>` | URL of the Clio server this server is connected to. |
| `closed_ledger` | `Option<serde_json::Value>` | Most recently closed ledger, when the server is not yet synced to validated. |
| `complete_ledgers` | `String` | Range(s) of ledger versions the server has locally, e.g. "63000000-63500000". |
| `hostid` | `String` | Human-readable hostname identifier for this node. |
| `io_latency_ms` | `u64` | Median I/O latency in milliseconds; high values indicate disk pressure. |
| `jq_trans_overflow` | `String` | Count of transactions dropped due to job queue overflow. |
| `last_close` | `ServerInfoLastClose` | Timing details from the most recent ledger close. |
| `load_factor` | `f64` | Current load factor relative to the base transaction cost. |
| `load_factor_fee_escalation` | `Option<f64>` | Current multiplier to the transaction cost to get into the open ledger. |
| `load_factor_fee_queue` | `Option<f64>` | Current multiplier to the transaction cost to get into the queue. |
| `load_factor_fee_reference` | `Option<f64>` | The load factor being used as a reference for fee calculation. |
| `load_factor_server` | `Option<f64>` | Current multiplier to the transaction cost based on load to the server. |
| `network_id` | `Option<u64>` | Network ID distinguishing mainnet from sidechains or testnets. |
| `peer_disconnects` | `String` | Total number of peer disconnects since startup. |
| `peer_disconnects_resources` | `String` | Peer disconnects caused by resource exhaustion. |
| `peers` | `u64` | Number of currently connected peers. |
| `ports` | `Option<Vec<ServerInfoPort>>` | Ports and protocols this server is listening on. |
| `pubkey_node` | `String` | Ed25519 public key identifying this node in the peer network. |
| `reporting` | `Option<ServerInfoReporting>` | Information about the reporting mode server. |
| `server_state` | `String` | Current server state, e.g. "full", "syncing", "connected". |
| `server_state_duration_us` | `String` | Time spent in the current server state, in microseconds. |
| `state_accounting` | `ServerInfoStateAccounting` | Per-state duration and transition counters since startup. |
| `time` | `String` | Current UTC time on the server. |
| `uptime` | `u64` | Server uptime in seconds. |
| `validated_ledger` | `Option<ServerInfoValidatedLedger>` | Most recently validated ledger summary; absent while syncing. |
| `validation_quorum` | `u64` | Minimum number of trusted validator votes required to validate a ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoReporting`

Reporting mode server details.

```rust
pub struct ServerInfoReporting {
    pub is_writer: bool,
    pub clio_server_url: Option<String>,
    pub etl_sources: Option<Vec<ServerInfoEtlSource>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `is_writer` | `bool` | Whether this server is a writer. |
| `clio_server_url` | `Option<String>` | The URL of the Clio server this server is connected to. |
| `etl_sources` | `Option<Vec<ServerInfoEtlSource>>` | Information about ETL sources. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoEtlSource`

ETL source details.

```rust
pub struct ServerInfoEtlSource {
    pub ip: String,
    pub port: u32,
    pub spec: String,
    pub validated: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ip` | `String` | The IP address of the ETL source. |
| `port` | `u32` | The port of the ETL source. |
| `spec` | `String` | The protocol specification. |
| `validated` | `bool` | Whether the source is validated. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoPort`

A port descriptor for a server_info response.

```rust
pub struct ServerInfoPort {
    pub port: String,
    pub protocol: Vec<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `port` | `String` | Port number string. |
| `protocol` | `Vec<String>` | Protocols served on this port. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoValidatedLedger`

Summary of the most recently validated ledger from `server_info`.

```rust
pub struct ServerInfoValidatedLedger {
    pub age: u64,
    pub base_fee_xrp: f64,
    pub hash: String,
    pub reserve_base_xrp: f64,
    pub reserve_inc_xrp: f64,
    pub seq: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `age` | `u64` | Seconds since this ledger was validated. |
| `base_fee_xrp` | `f64` | Reference transaction cost in XRP (not drops). |
| `hash` | `String` | Hash of the most recently validated ledger. |
| `reserve_base_xrp` | `f64` | Base account reserve in XRP. |
| `reserve_inc_xrp` | `f64` | Owner reserve increment per object in XRP. |
| `seq` | `u32` | Sequence number of the most recently validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoLastClose`

Timing information from the most recent ledger close.

```rust
pub struct ServerInfoLastClose {
    pub converge_time_s: f64,
    pub proposers: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `converge_time_s` | `f64` | Time the consensus round took to converge, in seconds. |
| `proposers` | `u32` | Number of trusted validators that participated in the consensus round. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoStateAccounting`

Per-state duration counters for a `server_info` response.

```rust
pub struct ServerInfoStateAccounting {
    pub connected: ServerInfoStateAccount,
    pub disconnected: ServerInfoStateAccount,
    pub full: ServerInfoStateAccount,
    pub syncing: ServerInfoStateAccount,
    pub tracking: ServerInfoStateAccount,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `connected` | `ServerInfoStateAccount` | Time and transitions spent in the "connected" state. |
| `disconnected` | `ServerInfoStateAccount` | Time and transitions spent in the "disconnected" state. |
| `full` | `ServerInfoStateAccount` | Time and transitions spent in the "full" (synced) state. |
| `syncing` | `ServerInfoStateAccount` | Time and transitions spent in the "syncing" state. |
| `tracking` | `ServerInfoStateAccount` | Time and transitions spent in the "tracking" state. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerInfoStateAccount`

Duration and transition count for a single server state.

```rust
pub struct ServerInfoStateAccount {
    pub duration_us: String,
    pub transitions: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `duration_us` | `String` | Total time spent in this state since startup, in microseconds. |
| `transitions` | `String` | Number of times the server entered this state. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `server_state`

Request and response types for the `server_state` command.

```rust
pub mod server_state { /* ... */ }
```

### Types

#### Struct `ServerStateRequest`

Retrieves a machine-readable summary of the server's state (values in drops, not XRP).

Prefer `server_state` over `server_info` when parsing programmatically, as numeric
fields use integer drops rather than floating-point XRP.

# Example
```rust
use xrpl::request::server_state::ServerStateRequest;

let request = ServerStateRequest::new().with_ledger_index("current");
```

```rust
pub struct ServerStateRequest {
    pub ledger_index: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_index` | `Option<serde_json::Value>` | Provide "current" to query a Clio server. |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Creates a new request with default options.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut. Use `"current"` to query a Clio server.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `ServerStateResult`

Response to a `server_state` request.

```rust
pub struct ServerStateResult {
    pub state: ServerState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `state` | `ServerState` | Detailed server state information. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerState`

**Attributes:**

- `Other("#[serde(default)]")`

Machine-readable server state details returned inside a `ServerStateResult`.

```rust
pub struct ServerState {
    pub amendment_blocked: Option<bool>,
    pub build_version: String,
    pub complete_ledgers: String,
    pub io_latency_ms: u64,
    pub jq_trans_overflow: String,
    pub last_close: LastClose,
    pub load_base: u64,
    pub load_factor: u64,
    pub load_factor_fee_escalation: u64,
    pub load_factor_fee_queue: u64,
    pub load_factor_fee_reference: u64,
    pub load_factor_server: u64,
    pub network_id: Option<u64>,
    pub peer_disconnects: String,
    pub peer_disconnects_resources: String,
    pub peers: u64,
    pub pubkey_node: String,
    pub server_state: String,
    pub server_state_duration_us: String,
    pub state_accounting: StateAccounting,
    pub time: String,
    pub uptime: u64,
    pub validated_ledger: Option<ValidatedLedger>,
    pub validation_quorum: u64,
    pub ports: Option<Vec<ServerStatePort>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amendment_blocked` | `Option<bool>` | Whether the server is blocked from participating due to unsupported amendments. |
| `build_version` | `String` | rippled build version string. |
| `complete_ledgers` | `String` | Range(s) of ledger versions the server has locally, e.g. "63000000-63500000". |
| `io_latency_ms` | `u64` | Median I/O latency in milliseconds; high values indicate disk pressure. |
| `jq_trans_overflow` | `String` | Count of transactions dropped due to job queue overflow. |
| `last_close` | `LastClose` | Timing details from the most recent ledger close. |
| `load_base` | `u64` | Reference load level (always 256). |
| `load_factor` | `u64` | Current load factor applied to the base transaction cost. |
| `load_factor_fee_escalation` | `u64` | Load factor from fee escalation for the open ledger. |
| `load_factor_fee_queue` | `u64` | Load factor applied to transactions held in the fee queue. |
| `load_factor_fee_reference` | `u64` | Fee reference load factor (usually 256). |
| `load_factor_server` | `u64` | Load factor from server-side resource constraints. |
| `network_id` | `Option<u64>` | Network ID distinguishing mainnet from sidechains or testnets. |
| `peer_disconnects` | `String` | Total number of peer disconnects since startup. |
| `peer_disconnects_resources` | `String` | Peer disconnects caused by resource exhaustion. |
| `peers` | `u64` | Number of currently connected peers. |
| `pubkey_node` | `String` | Ed25519 public key identifying this node in the peer network. |
| `server_state` | `String` | Current server state, e.g. "full", "syncing", "connected". |
| `server_state_duration_us` | `String` | Time spent in the current server state, in microseconds. |
| `state_accounting` | `StateAccounting` | Per-state duration and transition counters since startup. |
| `time` | `String` | Current UTC time on the server. |
| `uptime` | `u64` | Server uptime in seconds. |
| `validated_ledger` | `Option<ValidatedLedger>` | Most recently validated ledger summary. |
| `validation_quorum` | `u64` | Minimum number of trusted validator votes required to validate a ledger. |
| `ports` | `Option<Vec<ServerStatePort>>` | Ports and protocols this server is listening on. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ServerStatePort`

**Attributes:**

- `Other("#[serde(default)]")`

A port descriptor for a server_state response.

```rust
pub struct ServerStatePort {
    pub port: String,
    pub protocol: Vec<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `port` | `String` | Port number string. |
| `protocol` | `Vec<String>` | Protocols served on this port. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `LastClose`

**Attributes:**

- `Other("#[serde(default)]")`

Timing information from the most recent ledger close (server_state variant).

```rust
pub struct LastClose {
    pub converge_time: u64,
    pub proposers: u64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `converge_time` | `u64` | Time the consensus round took to converge, in milliseconds. |
| `proposers` | `u64` | Number of trusted validators that participated in the consensus round. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `StateAccounting`

**Attributes:**

- `Other("#[serde(default)]")`

Per-state duration counters for a `server_state` response.

```rust
pub struct StateAccounting {
    pub connected: AccountingState,
    pub disconnected: AccountingState,
    pub full: AccountingState,
    pub syncing: AccountingState,
    pub tracking: AccountingState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `connected` | `AccountingState` | Time and transitions spent in the "connected" state. |
| `disconnected` | `AccountingState` | Time and transitions spent in the "disconnected" state. |
| `full` | `AccountingState` | Time and transitions spent in the "full" (synced) state. |
| `syncing` | `AccountingState` | Time and transitions spent in the "syncing" state. |
| `tracking` | `AccountingState` | Time and transitions spent in the "tracking" state. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountingState`

**Attributes:**

- `Other("#[serde(default)]")`

Duration and transition count for a single server state.

```rust
pub struct AccountingState {
    pub duration_us: String,
    pub transitions: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `duration_us` | `String` | Total time spent in this state since startup, in microseconds. |
| `transitions` | `String` | Number of times the server entered this state. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `ValidatedLedger`

**Attributes:**

- `Other("#[serde(default)]")`

Summary of the most recently validated ledger from `server_state` (values in drops).

```rust
pub struct ValidatedLedger {
    pub age: u64,
    pub base_fee: u64,
    pub close_time: u64,
    pub hash: String,
    pub reserve_base: u64,
    pub reserve_inc: u64,
    pub seq: u64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `age` | `u64` | Seconds since this ledger was validated. |
| `base_fee` | `u64` | Reference transaction cost in drops. |
| `close_time` | `u64` | Close time as Ripple epoch seconds. |
| `hash` | `String` | Hash of the most recently validated ledger. |
| `reserve_base` | `u64` | Base account reserve in drops. |
| `reserve_inc` | `u64` | Owner reserve increment per object in drops. |
| `seq` | `u64` | Sequence number of the most recently validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `submit`

Request and response types for the `submit` command.

```rust
pub mod submit { /* ... */ }
```

### Types

#### Struct `SubmitRequest`

Submits a signed transaction blob to the XRPL network.

The `tx_blob` must be a fully signed, hex-encoded transaction.
Always verify the result using `validated` status, not just the submission engine code.

# Example
```rust
use xrpl::request::submit::SubmitRequest;

let request = SubmitRequest::new("1200002200000000...").with_fail_hard(true);
```

```rust
pub struct SubmitRequest {
    pub tx_blob: String,
    pub fail_hard: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `tx_blob` | `String` | Hex-encoded signed transaction blob. |
| `fail_hard` | `Option<bool>` | If true, reject the transaction instead of queuing it when it cannot enter the open ledger. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(tx_blob: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request with the given signed transaction hex blob.

- ```rust
  pub fn with_fail_hard(self: Self, value: bool) -> Self { /* ... */ }
  ```
  Rejects the transaction instead of queuing it when it cannot enter the open ledger.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `SubmitResponse`

Response to a `submit` request.

```rust
pub struct SubmitResponse {
    pub engine_result: String,
    pub engine_result_code: i64,
    pub engine_result_message: String,
    pub tx_blob: Option<String>,
    pub tx_json: Option<serde_json::Value>,
    pub accepted: Option<bool>,
    pub account_sequence_available: Option<u32>,
    pub account_sequence_next: Option<u32>,
    pub applied: Option<bool>,
    pub broadcast: Option<bool>,
    pub kept: Option<bool>,
    pub queued: Option<bool>,
    pub open_ledger_cost: Option<String>,
    pub validated_ledger_index: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `engine_result` | `String` | Symbolic result code, e.g. "tesSUCCESS" or "tecNO_DST". |
| `engine_result_code` | `i64` | Numeric result code corresponding to `engine_result`. |
| `engine_result_message` | `String` | Human-readable description of the result. |
| `tx_blob` | `Option<String>` | Hex-encoded transaction blob as received. |
| `tx_json` | `Option<serde_json::Value>` | The complete transaction in JSON format. |
| `accepted` | `Option<bool>` | Whether the transaction was accepted by the server's processing engine. |
| `account_sequence_available` | `Option<u32>` | Next sequence number that could be used without a gap. |
| `account_sequence_next` | `Option<u32>` | Next sequence number that will be consumed. |
| `applied` | `Option<bool>` | Whether the transaction was applied to the open ledger. |
| `broadcast` | `Option<bool>` | Whether the transaction was broadcast to peers. |
| `kept` | `Option<bool>` | Whether the transaction was kept in the queue or ledger. |
| `queued` | `Option<bool>` | Whether the transaction was placed in the fee queue. |
| `open_ledger_cost` | `Option<String>` | Minimum fee in drops required to enter the current open ledger. |
| `validated_ledger_index` | `Option<u32>` | Sequence number of the most recently validated ledger at submission time. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `submit_multisigned`

Request and response types for the `submit_multisigned` command.

```rust
pub mod submit_multisigned { /* ... */ }
```

### Types

#### Struct `SubmitMultisignedRequest`

Submits a multi-signed transaction to the network.

Use `submit` for single-signed transactions.

# Example
```rust
use xrpl::request::submit_multisigned::SubmitMultisignedRequest;
use serde_json::json;

let request = SubmitMultisignedRequest::new(json!({"TransactionType": "Payment"}))
    .with_fail_hard(true);
```

```rust
pub struct SubmitMultisignedRequest {
    pub tx_json: serde_json::Value,
    pub fail_hard: Option<bool>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `tx_json` | `serde_json::Value` | The fully assembled and multi-signed transaction as a JSON object. |
| `fail_hard` | `Option<bool>` | If true, reject the transaction instead of queuing it when it cannot enter the open ledger. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl Into<Value>: Into<Value>>(tx_json: impl Into<Value>) -> Self { /* ... */ }
  ```
  Creates a new request with the given transaction JSON.

- ```rust
  pub fn with_fail_hard(self: Self, value: bool) -> Self { /* ... */ }
  ```
  Rejects the transaction instead of queuing it when it cannot enter the open ledger.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `SubmitMultisignedResponse`

Response to a `submit_multisigned` request.

```rust
pub struct SubmitMultisignedResponse {
    pub engine_result: String,
    pub engine_result_code: i64,
    pub engine_result_message: String,
    pub tx_blob: Option<String>,
    pub accepted: Option<bool>,
    pub account_sequence_available: Option<u32>,
    pub account_sequence_next: Option<u32>,
    pub applied: Option<bool>,
    pub broadcast: Option<bool>,
    pub kept: Option<bool>,
    pub queued: Option<bool>,
    pub open_ledger_cost: Option<String>,
    pub validated_ledger_index: Option<u32>,
    pub tx_json: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `engine_result` | `String` | Symbolic result code, e.g. "tesSUCCESS" or "tecNO_DST". |
| `engine_result_code` | `i64` | Numeric result code corresponding to `engine_result`. |
| `engine_result_message` | `String` | Human-readable description of the result. |
| `tx_blob` | `Option<String>` | Hex-encoded transaction blob as received. |
| `accepted` | `Option<bool>` | Whether the transaction was accepted by the server's processing engine. |
| `account_sequence_available` | `Option<u32>` | Next sequence number that could be used without a gap. |
| `account_sequence_next` | `Option<u32>` | Next sequence number that will be consumed. |
| `applied` | `Option<bool>` | Whether the transaction was applied to the open ledger. |
| `broadcast` | `Option<bool>` | Whether the transaction was broadcast to peers. |
| `kept` | `Option<bool>` | Whether the transaction was kept in the queue or ledger. |
| `queued` | `Option<bool>` | Whether the transaction was placed in the fee queue. |
| `open_ledger_cost` | `Option<String>` | Minimum fee in drops required to enter the current open ledger. |
| `validated_ledger_index` | `Option<u32>` | Sequence number of the most recently validated ledger at submission time. |
| `tx_json` | `Option<serde_json::Value>` | The complete transaction in JSON format. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `transaction_entry`

Request and response types for the `transaction_entry` command.

```rust
pub mod transaction_entry { /* ... */ }
```

### Types

#### Struct `TransactionEntryRequest`

Retrieves information on a transaction that is included in a specific ledger.

Unlike `tx`, this always searches a specific ledger version rather than
scanning the ledger history.

```rust
pub struct TransactionEntryRequest {
    pub tx_hash: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<serde_json::Value>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `tx_hash` | `String` | Hash of the transaction to look up. |
| `ledger_hash` | `Option<String>` | Ledger hash to target a specific ledger version. |
| `ledger_index` | `Option<serde_json::Value>` | Ledger index or shortcut ("validated", "closed", "current"). |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(tx_hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new request for the given transaction hash.

- ```rust
  pub fn with_ledger_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, ledger_hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the target ledger by its hash.

- ```rust
  pub fn with_ledger_index</* synthetic */ impl Into<Value>: Into<Value>>(self: Self, ledger_index: impl Into<Value>) -> Self { /* ... */ }
  ```
  Sets the ledger index or shortcut ("validated", "closed", "current").

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `TransactionEntryResponse`

Response to a `transaction_entry` request.

```rust
pub struct TransactionEntryResponse {
    pub ledger_index: u32,
    pub ledger_hash: String,
    pub tx_json: crate::types::Transaction,
    pub metadata: serde_json::Value,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_index` | `u32` | The transaction in JSON format. |
| `ledger_hash` | `String` |  |
| `tx_json` | `crate::types::Transaction` |  |
| `metadata` | `serde_json::Value` | Execution metadata, including `delivered_amount` and affected nodes. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `tx`

Request and response types for the `tx` command.

```rust
pub mod tx { /* ... */ }
```

### Types

#### Struct `TxRequest`

Looks up a single transaction by its hash or Compact Transaction Identifier (CTID).

Use `tx_hash` for most lookups. Use `ctid` when you have a compact reference
from a validator or receipt. Provide `min_ledger`/`max_ledger` to narrow the
search range and reduce server-side cost.

# Example
```rust
use xrpl::request::tx::TxRequest;

let request = TxRequest::by_hash("E08D6E9754025BA2534A78707605E0601F03ACE063687A0CA1BDDACFCD1698C7")
    .with_min_ledger(1000)
    .with_max_ledger(2000);
```

```rust
pub struct TxRequest {
    pub tx_hash: Option<String>,
    pub ctid: Option<String>,
    pub binary: Option<bool>,
    pub min_ledger: Option<u32>,
    pub max_ledger: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `tx_hash` | `Option<String>` | Transaction hash (64-character hex). |
| `ctid` | `Option<String>` | Compact Transaction Identifier (alternative to `tx_hash`). |
| `binary` | `Option<bool>` | If true, return the transaction in binary format. |
| `min_ledger` | `Option<u32>` | Earliest ledger sequence to search (inclusive). |
| `max_ledger` | `Option<u32>` | Latest ledger sequence to search (inclusive). |

##### Implementations

###### Methods

- ```rust
  pub fn by_hash</* synthetic */ impl AsRef<str>: AsRef<str>>(hash: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request to look up a transaction by its 64-character hex hash.

- ```rust
  pub fn by_ctid</* synthetic */ impl AsRef<str>: AsRef<str>>(ctid: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a request to look up a transaction by its Compact Transaction Identifier.

- ```rust
  pub fn with_binary(self: Self, binary: bool) -> Self { /* ... */ }
  ```
  Returns the transaction as a binary blob instead of JSON.

- ```rust
  pub fn with_min_ledger(self: Self, min_ledger: u32) -> Self { /* ... */ }
  ```
  Sets the earliest ledger sequence to search (inclusive).

- ```rust
  pub fn with_max_ledger(self: Self, max_ledger: u32) -> Self { /* ... */ }
  ```
  Sets the latest ledger sequence to search (inclusive).

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
#### Struct `TxResponse`

Response to a `tx` request.

```rust
pub struct TxResponse {
    pub transaction: Option<crate::types::Transaction>,
    pub ctid: Option<String>,
    pub hash: Option<String>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub meta: Option<crate::types::TransactionMeta>,
    pub date: Option<u32>,
    pub validated: bool,
    pub in_ledger: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `transaction` | `Option<crate::types::Transaction>` |  |
| `ctid` | `Option<String>` | Compact Transaction Identifier, if available. |
| `hash` | `Option<String>` | Transaction hash. |
| `ledger_hash` | `Option<String>` | Hash of the ledger version that contains this transaction. |
| `ledger_index` | `Option<u32>` | Sequence number of the ledger version that contains this transaction. |
| `meta` | `Option<crate::types::TransactionMeta>` | Execution metadata, including `delivered_amount` and affected nodes. |
| `date` | `Option<u32>` | Close time of the ledger in which the transaction was applied. |
| `validated` | `bool` | Whether the transaction is in a validated ledger. |
| `in_ledger` | `Option<u32>` | The ledger index of the ledger that includes this transaction. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **HasTransactionMeta**
  - ```rust
    fn transaction_meta(self: &Self) -> Option<&TransactionMeta> { /* ... */ }
    ```

### Types

#### Enum `XrplResponse`

**Attributes:**

- `Other("#[serde(untagged)]")`

Top-level envelope for every response from the rippled server.

Deserializes into either [`XrplResponse::Success`] (carrying the typed result) or
[`XrplResponse::Error`] (carrying the XRPL error code and message). Use
[`XrplResponse::result`] to convert into a standard [`Result`].

# Examples

```rust
use xrpl::request::XrplResponse;
use xrpl::request::account_info::AccountInfoResponse;

async fn handle(resp: XrplResponse<AccountInfoResponse>) {
    match resp.result() {
        Ok(info) => println!("{}", info.account_data.balance),
        Err(e) => eprintln!("XRPL error: {e}"),
    }
}
```

```rust
pub enum XrplResponse<T> {
    Success {
        id: Option<serde_json::Value>,
        result: T,
        kind: String,
        status: String,
    },
    Error {
        id: Option<serde_json::Value>,
        error: String,
        error_code: Option<i32>,
        error_message: Option<String>,
        request: Option<serde_json::Value>,
        kind: String,
        status: String,
    },
}
```

##### Variants

###### `Success`

The server processed the request successfully; `result` holds the typed payload.

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `id` | `Option<serde_json::Value>` | Optional correlation id echoed from the request. |
| `result` | `T` | Typed response payload. |
| `kind` | `String` | Response type label (e.g. `"response"`); wire field `type`. |
| `status` | `String` | Always `"success"` for this variant. |

###### `Error`

The server returned an error; inspect `error` and `error_message` for details.

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `id` | `Option<serde_json::Value>` | Optional correlation id echoed from the request. |
| `error` | `String` | Short XRPL error token (e.g. `"invalidParams"`). |
| `error_code` | `Option<i32>` | Numeric XRPL error code. |
| `error_message` | `Option<String>` | Human-readable error description. |
| `request` | `Option<serde_json::Value>` | Echo of the original request that triggered the error. |
| `kind` | `String` | Response type label; wire field `type`. |
| `status` | `String` | Always `"error"` for this variant. |

##### Implementations

###### Methods

- ```rust
  pub fn result(self: Self) -> Result<T, XrplError> { /* ... */ }
  ```
  Converts this response envelope into a [`Result`], returning the typed

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

### Traits

#### Trait `XrplRequest`

Implemented by every XRPL request type, providing the command name, API version,
and a uniform way to serialize the request into a JSON value ready for submission
over a WebSocket or JSON-RPC connection.

# Examples

```rust
use xrpl::request::XrplRequest;
use xrpl::request::account_info::AccountInfoRequest;

let req = AccountInfoRequest::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
let json = req.to_value().expect("request must be serializable");
assert_eq!(json["command"], "account_info");
```

```rust
pub trait XrplRequest: Serialize {
    /* Associated items */
}
```

> This trait is not object-safe and cannot be used in dynamic trait objects.

##### Required Items

###### Associated Types

- `Response`: Deserialized server response type for this request.

###### Associated Constants

- `COMMAND`: XRPL WebSocket/JSON-RPC command name (e.g. `"account_info"`).

##### Provided Methods

- ```rust
  fn to_value(self: &Self) -> Result<Value, serde_json::Error> { /* ... */ }
  ```
  Serializes the request into a [`serde_json::Value`] with `command` and

##### Implementations

This trait is implemented for the following types:

- `AccountChannelsRequest`
- `AccountCurrenciesRequest`
- `AccountInfoRequest`
- `AccountLinesRequest`
- `AccountNftsRequest`
- `AccountObjectsRequest`
- `AccountOffersRequest`
- `AccountTxRequest`
- `AmmInfoRequest`
- `BookOffersRequest`
- `FeeRequest`
- `LedgerRequest`
- `LedgerClosedRequest`
- `LedgerCurrentRequest`
- `LedgerDataRequest`
- `LedgerEntryRequest`
- `NftBuyOffersRequest`
- `NftSellOffersRequest`
- `RipplePathFindRequest`
- `ServerInfoRequest`
- `ServerStateRequest`
- `SubmitRequest`
- `SubmitMultisignedRequest`
- `TransactionEntryRequest`
- `TxRequest`
- `AccountTransactionsSubscription`
- `BookSubscription`
- `BookChangesSubscription`
- `LedgerSubscription`
- `TransactionsSubscription`

#### Trait `XrplSubscription`

Extends [`XrplRequest`] for commands that open a persistent subscription and push
server-initiated messages (e.g. `subscribe` for ledger or transaction streams).

The associated `Message` type is the deserialized form of each pushed event.

Subscriptions that use the `streams` wire field (e.g. `ledger`, `validations`,
`consensus`) should override `STREAM` with their stream name. Subscriptions
that use other fields (e.g. `accounts`, `books`) leave it at the default.

```rust
pub trait XrplSubscription: XrplRequest {
    /* Associated items */
}
```

> This trait is not object-safe and cannot be used in dynamic trait objects.

##### Required Items

###### Associated Types

- `Message`: The type of each streaming message delivered after the subscription is opened.

###### Associated Constants

- `MESSAGE_TYPE`: Wire `"type"` tag carried by this subscription's push messages

##### Provided Methods

##### Implementations

This trait is implemented for the following types:

- `AccountTransactionsSubscription`
- `BookSubscription`
- `BookChangesSubscription`
- `LedgerSubscription`
- `TransactionsSubscription`

## Module `session`

Subscription session for receiving streamed messages.

```rust
pub mod session { /* ... */ }
```

### Types

#### Enum `SubscriptionEvent`

**Attributes:**

- `Other("#[allow(clippy::large_enum_variant)]")`
- `NonExhaustive`

Unified event over every subscription stream type, dispatched on the wire
`"type"` field. Lets a single [`SubscriptionSession`] handle all events in
one match loop.

Unrecognized `"type"` values deserialize into [`Unknown`](Self::Unknown).
A recognized `"type"` with a body that doesn't match its variant's shape
fails deserialization instead of falling back to `Unknown`.

```rust
pub enum SubscriptionEvent {
    Ledger(crate::subscriptions::LedgerMessage),
    BookChanges(crate::subscriptions::BookChangesMessage),
    Transaction(crate::subscriptions::AccountTransactionMessage),
    Unknown {
        message_type: String,
        value: serde_json::Value,
    },
}
```

##### Variants

###### `Ledger`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `crate::subscriptions::LedgerMessage` |  |

###### `BookChanges`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `crate::subscriptions::BookChangesMessage` |  |

###### `Transaction`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `crate::subscriptions::AccountTransactionMessage` |  |

###### `Unknown`

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `message_type` | `String` |  |
| `value` | `serde_json::Value` |  |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as >::Error>
where
    D: serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `SubscriptionStream`

A type-scoped receiver over a shared subscription connection, scoped to
one subscription's message type (`SubscriptionStream<LedgerMessage>`) or
unified over all of a session's subscriptions (`SubscriptionStream<SubscriptionEvent>`).

Independently owned via its own `connection` sender clone - keeps working
after the [`SubscriptionSession`] that created it is dropped. Drop it to
stop locally, or call [`unsubscribe`](Self::unsubscribe) to also stop the
server side.

```rust
pub struct SubscriptionStream<T> {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub async fn recv(self: &mut Self) -> Result<T, XrplError> { /* ... */ }
  ```
  Receive the next message from this stream.

- ```rust
  pub async fn unsubscribe(self: Self) -> Result<(), XrplError> { /* ... */ }
  ```
  Tells the server to stop this subscription and awaits its

###### Trait Implementations

- **Drop**
  - ```rust
    fn drop(self: &mut Self) { /* ... */ }
    ```

#### Struct `SubscriptionSession`

Session over a shared connection on which subscription streams can be
opened via [`subscribe`](Self::subscribe).

Each derived [`SubscriptionStream`] is independently owned; dropping this
session only gives up the ability to open further subscriptions, it does
not affect streams already handed out. Manage each stream's own lifecycle
via [`unsubscribe`](SubscriptionStream::unsubscribe) or by dropping it.

```rust
pub struct SubscriptionSession<T = SubscriptionEvent> {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub async fn recv(self: &mut Self) -> Result<T, XrplError> { /* ... */ }
  ```
  Receive the next message from this session's stream.

- ```rust
  pub async fn subscribe<U>(self: &mut Self, sub: &U) -> Result<(<U as >::Response, SubscriptionStream<<U as >::Message>), XrplError>
where
    U: XrplSubscription,
    <U as >::Message: Clone + Send + DeserializeOwned + Debug + ''static { /* ... */ }
  ```
  Open an additional subscription stream over this session's shared connection.

###### Trait Implementations

## Module `subscriptions`

Subscription request types and streamed message types.

```rust
pub mod subscriptions { /* ... */ }
```

### Modules

## Module `account_tx`

Account-transaction subscription types and streamed messages.

```rust
pub mod account_tx { /* ... */ }
```

### Types

#### Struct `AccountTransactionsSubscription`

Subscription request for account transaction events.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::AccountTransactionsSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let sub = AccountTransactionsSubscription::validated(
        ["rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"],
    )?;
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;
    while let Ok(msg) = stream.recv().await {
        println!("{}: {}", msg.hash, msg.engine_result);
    }
    Ok(())
}
```

```rust
pub struct AccountTransactionsSubscription {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn proposed<I, S>(accounts: I) -> Result<Self, BuildError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str> { /* ... */ }
  ```
  Subscribe to `accounts_proposed`: validated transactions plus in-flight

- ```rust
  pub fn validated<I, S>(accounts: I) -> Result<Self, BuildError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str> { /* ... */ }
  ```
  Subscribe to `accounts`: validated transactions only.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
- **XrplSubscription**
#### Struct `AccountSubscriptionResponse`

Initial response returned when subscribing to account transaction events.

```rust
pub struct AccountSubscriptionResponse {
    pub accounts: Option<Vec<String>>,
    pub accounts_proposed: Option<Vec<String>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `accounts` | `Option<Vec<String>>` | Accounts enrolled in the validated-transactions stream, when applicable. |
| `accounts_proposed` | `Option<Vec<String>>` | Accounts enrolled in the proposed-transactions stream, when applicable. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `AccountTransactionMessage`

A server-pushed message for a transaction that affects a subscribed account.

Received on both the `accounts` and `accounts_proposed` streams. The
`validated` flag distinguishes whether the transaction is in a closed,
immutable ledger (`true`) or is still in-flight (`false`).

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::AccountTransactionsSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let sub = AccountTransactionsSubscription::validated(
        ["rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe"],
    )?;
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;
    while let Ok(msg) = stream.recv().await {
        if msg.validated {
            println!("{}: {}", msg.hash, msg.engine_result);
        }
    }
    Ok(())
}
```

```rust
pub struct AccountTransactionMessage {
    pub close_time_iso: Option<String>,
    pub engine_result: String,
    pub engine_result_code: i32,
    pub engine_result_message: String,
    pub hash: String,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<u32>,
    pub ledger_current_index: Option<u32>,
    pub meta: Option<crate::types::TransactionMeta>,
    pub tx_json: crate::types::Transaction,
    pub validated: bool,
    pub ctid: Option<String>,
    pub status: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `close_time_iso` | `Option<String>` | ISO 8601 close time of the ledger, when available. |
| `engine_result` | `String` | Transaction result code (e.g. `"tesSUCCESS"`, `"tecNO_DST"`). |
| `engine_result_code` | `i32` | Numeric form of the engine result code. |
| `engine_result_message` | `String` | Human-readable description of the engine result. |
| `hash` | `String` | SHA-512Half hash that uniquely identifies the transaction. |
| `ledger_hash` | `Option<String>` | Hash of the validated ledger that contains this transaction, when validated. |
| `ledger_index` | `Option<u32>` | Sequence number of the validated ledger that contains this transaction. |
| `ledger_current_index` | `Option<u32>` | Sequence number of the current open ledger (present when not yet validated). |
| `meta` | `Option<crate::types::TransactionMeta>` | Transaction metadata with affected nodes and delivered amount, when validated. |
| `tx_json` | `crate::types::Transaction` | The full transaction object. |
| `validated` | `bool` | `true` if the transaction is in a closed, immutable ledger. |
| `ctid` | `Option<String>` | Compact Transaction Identifier for cross-network lookup, when present. |
| `status` | `Option<String>` | Internal submission status string, when present. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **HasTransactionMeta**
  - ```rust
    fn transaction_meta(self: &Self) -> Option<&TransactionMeta> { /* ... */ }
    ```

## Module `book`

Order-book subscription types and streamed messages.

```rust
pub mod book { /* ... */ }
```

### Types

#### Struct `BookSubscription`

Subscription request for order book updates on the XRPL.

The `books` stream sends a transaction message whenever a transaction
affects a subscribed order book - identical in format to the `transactions`
stream messages.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::BookSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let sub = BookSubscription::xrp_to_issued_currency(
        "USD",
        "rvYAfWj5gh67oV6fW32ZzP3Aw4Eubs59B",
        false,
    )?;
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;
    while let Ok(msg) = stream.recv().await {
        println!("{}: {}", msg.hash, msg.engine_result);
    }
    Ok(())
}
```

```rust
pub struct BookSubscription {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Create an empty subscription; add books with [`with_book`](Self::with_book) or [`with_books`](Self::with_books).

- ```rust
  pub fn with_book</* synthetic */ impl Into<Book>: Into<Book>>(self: Self, book: impl Into<Book>) -> Self { /* ... */ }
  ```
  Add a single book to the subscription.

- ```rust
  pub fn with_books<I, B>(self: Self, books: I) -> Self
where
    I: IntoIterator<Item = B>,
    B: Into<Book> { /* ... */ }
  ```
  Add multiple books to the subscription.

- ```rust
  pub fn xrp_to_issued_currency(currency: &str, issuer: &str, snapshot: bool) -> Result<Self, BuildError> { /* ... */ }
  ```
  Subscribe to an XRP-to-issued-currency order book (e.g. XRP/USD).

- ```rust
  pub fn currency_pair(gets_currency: &str, gets_issuer: Option<&str>, pays_currency: &str, pays_issuer: Option<&str>, snapshot: bool, both: bool) -> Result<Self, BuildError> { /* ... */ }
  ```
  Subscribe to any currency pair order book.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
- **XrplSubscription**
#### Struct `Book`

A single order book (currency pair) to subscribe to.

```rust
pub struct Book {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn xrp_to_issued_currency(currency: &str, issuer: &str, snapshot: bool) -> Result<Self, BuildError> { /* ... */ }
  ```
  Create a book for XRP to an issued currency (e.g. XRP/USD).

- ```rust
  pub fn issued_currency_to_xrp(currency: &str, issuer: &str, snapshot: bool) -> Result<Self, BuildError> { /* ... */ }
  ```
  Create a book for an issued currency to XRP (e.g. USD/XRP).

- ```rust
  pub fn currency_pair(gets_currency: &str, gets_issuer: Option<&str>, pays_currency: &str, pays_issuer: Option<&str>, snapshot: bool, both: bool) -> Result<Self, BuildError> { /* ... */ }
  ```
  Create a book for any currency pair.

- ```rust
  pub fn both_sides(self: Self) -> Self { /* ... */ }
  ```
  Subscribe to both sides (buy and sell) of the order book.

- ```rust
  pub fn with_snapshot(self: Self) -> Self { /* ... */ }
  ```
  Include a snapshot of the current order book state on subscribe.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `BookCurrency`

Currency and optional issuer for one side of an order book.

```rust
pub struct BookCurrency {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `BookSubscriptionResponse`

Initial response returned when subscribing to an order book.

When `snapshot: true` is set, the response also includes an `offers` array
with the current order book state, delivered as part of the subscribe response.

```rust
pub struct BookSubscriptionResponse {
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `book_changes`

Aggregated order-book change subscription and streamed messages.

```rust
pub mod book_changes { /* ... */ }
```

### Types

#### Struct `BookChangesSubscription`

Subscription request for the `book_changes` stream.

Sends a `bookChanges` message on every validated ledger close, containing
a summary of all order book changes that occurred in that ledger.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::BookChangesSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&BookChangesSubscription::default()).await?;
    while let Ok(msg) = stream.recv().await {
        println!("ledger {} had {} book changes", msg.ledger_index, msg.changes.len());
    }
    Ok(())
}
```

```rust
pub struct BookChangesSubscription {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
- **XrplSubscription**
#### Struct `BookChangesSubscriptionResponse`

Initial response returned when subscribing to the `book_changes` stream.

```rust
pub struct BookChangesSubscriptionResponse {
    pub fee_base: Option<i64>,
    pub ledger_hash: Option<String>,
    pub ledger_index: Option<i64>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `fee_base` | `Option<i64>` | Base transaction fee in fee units at the time of subscription. |
| `ledger_hash` | `Option<String>` | Hash of the most recently validated ledger at the time of subscription. |
| `ledger_index` | `Option<i64>` | Sequence number of the most recently validated ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `BookChangesMessage`

A `bookChanges` stream message, emitted on every validated ledger close.

```rust
pub struct BookChangesMessage {
    pub ledger_index: u64,
    pub ledger_hash: String,
    pub ledger_time: u64,
    pub changes: Vec<BookUpdate>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ledger_index` | `u64` | Sequence number of the closed ledger. |
| `ledger_hash` | `String` | Hash of the closed ledger. |
| `ledger_time` | `u64` | Close time of the ledger in seconds since the Ripple epoch. |
| `changes` | `Vec<BookUpdate>` | One entry for each order book that had activity in this ledger. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `BookUpdate`

One entry per order book that changed in the ledger.

`currency_a` and `currency_b` identify the pair as `"XRP_drops"` for XRP
or `"issuer/currency"` for issued currencies. All numeric fields are
string-encoded to preserve precision.

```rust
pub struct BookUpdate {
    pub currency_a: String,
    pub currency_b: String,
    pub volume_a: String,
    pub volume_b: String,
    pub high: String,
    pub low: String,
    pub open: String,
    pub close: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `currency_a` | `String` | First asset in the pair (`"XRP_drops"` for XRP, `"issuer/currency"` for tokens). |
| `currency_b` | `String` | Second asset in the pair. |
| `volume_a` | `String` | Total amount of `currency_a` traded. |
| `volume_b` | `String` | Total amount of `currency_b` traded. |
| `high` | `String` | Highest exchange rate seen in this ledger (currency_a per currency_b). |
| `low` | `String` | Lowest exchange rate seen in this ledger. |
| `open` | `String` | Opening exchange rate (first trade in this ledger). |
| `close` | `String` | Closing exchange rate (last trade in this ledger). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `ledger`

Ledger-close subscription types and streamed messages.

```rust
pub mod ledger { /* ... */ }
```

### Types

#### Struct `LedgerSubscription`

Subscription request for the `ledger` stream.

Sends a `ledgerClosed` message whenever the consensus process declares
a new validated ledger.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::LedgerSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&LedgerSubscription::new()).await?;
    while let Ok(msg) = stream.recv().await {
        println!("ledger {} closed ({} txns)", msg.ledger_index, msg.txn_count);
    }
    Ok(())
}
```

```rust
pub struct LedgerSubscription {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
- **XrplSubscription**
#### Struct `LedgerSubscriptionResponse`

Initial response when subscribing to the `ledger` stream.

Contains the same fields as [`LedgerMessage`], except `type` and `txn_count`.

```rust
pub struct LedgerSubscriptionResponse {
    pub fee_base: i64,
    pub fee_ref: Option<i64>,
    pub ledger_hash: String,
    pub ledger_index: i64,
    pub ledger_time: i64,
    pub network_id: Option<u32>,
    pub reserve_base: i64,
    pub reserve_inc: i64,
    pub validated_ledgers: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `fee_base` | `i64` | Base transaction fee in fee units. |
| `fee_ref` | `Option<i64>` | Fee units per transaction cost unit; omitted when XRPFees amendment is active. |
| `ledger_hash` | `String` | Hash of the most recently validated ledger. |
| `ledger_index` | `i64` | Sequence number of the most recently validated ledger. |
| `ledger_time` | `i64` | Close time of the most recently validated ledger (seconds since Ripple epoch). |
| `network_id` | `Option<u32>` | Network ID that identifies the XRPL network, when present. |
| `reserve_base` | `i64` | Minimum XRP reserve for an account, in drops. |
| `reserve_inc` | `i64` | Additional XRP reserve per owned ledger object, in drops. |
| `validated_ledgers` | `Option<String>` | Comma-separated ranges of ledger sequence numbers available on this server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `LedgerMessage`

A `ledgerClosed` stream message, emitted on every validated ledger close.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::LedgerSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&LedgerSubscription::new()).await?;
    while let Ok(msg) = stream.recv().await {
        println!("ledger {} closed ({} txns)", msg.ledger_index, msg.txn_count);
    }
    Ok(())
}
```

```rust
pub struct LedgerMessage {
    pub fee_base: i64,
    pub fee_ref: Option<i64>,
    pub ledger_hash: String,
    pub ledger_index: i64,
    pub ledger_time: i64,
    pub network_id: Option<u32>,
    pub reserve_base: i64,
    pub reserve_inc: i64,
    pub txn_count: i64,
    pub validated_ledgers: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `fee_base` | `i64` | Base transaction fee in fee units. |
| `fee_ref` | `Option<i64>` | Omitted when the XRPFees amendment is enabled. |
| `ledger_hash` | `String` | Hash of the closed ledger. |
| `ledger_index` | `i64` | Sequence number of the closed ledger. |
| `ledger_time` | `i64` | Close time of the ledger in seconds since the Ripple epoch. |
| `network_id` | `Option<u32>` | Network ID that identifies the XRPL network, when present. |
| `reserve_base` | `i64` | Minimum XRP reserve for an account, in drops. |
| `reserve_inc` | `i64` | Additional XRP reserve per owned ledger object, in drops. |
| `txn_count` | `i64` | Number of transactions included in the closed ledger. |
| `validated_ledgers` | `Option<String>` | Comma-separated ranges of ledger sequence numbers available on this server. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `transaction`

Transaction stream subscription types and streamed messages.

```rust
pub mod transaction { /* ... */ }
```

### Types

#### Struct `TransactionsSubscription`

Subscription request for all transaction stream events.

Use [`validated`](Self::validated) (default) for confirmed transactions or
[`proposed`](Self::proposed) to also receive in-flight transactions.

# Examples

```no_run
use xrpl::Client;
use xrpl::subscriptions::TransactionsSubscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&TransactionsSubscription::validated()).await?;
    while let Ok(msg) = stream.recv().await {
        println!("{}: {}", msg.hash, msg.engine_result);
    }
    Ok(())
}
```

```rust
pub struct TransactionsSubscription {
    // Some fields omitted
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| *private fields* | ... | *Some fields have been omitted* |

##### Implementations

###### Methods

- ```rust
  pub fn proposed() -> Self { /* ... */ }
  ```
  Subscribe to the `transactions_proposed` stream: all validated transactions

- ```rust
  pub fn validated() -> Self { /* ... */ }
  ```
  Subscribe to the `transactions` stream: validated transactions only.

###### Trait Implementations

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **XrplRequest**
- **XrplSubscription**
#### Struct `TransactionsSubscriptionResponse`

Initial response returned when subscribing to the transactions stream.

```rust
pub struct TransactionsSubscriptionResponse {
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

### Re-exports

#### Re-export `account_tx::*`

```rust
pub use account_tx::*;
```

#### Re-export `book::*`

```rust
pub use book::*;
```

#### Re-export `book_changes::*`

```rust
pub use book_changes::*;
```

#### Re-export `ledger::*`

```rust
pub use ledger::*;
```

#### Re-export `transaction::*`

```rust
pub use transaction::*;
```

## Module `time`

Ripple-epoch time conversion utilities.
Ripple-epoch time utilities.

The XRP Ledger measures time in seconds since the **Ripple Epoch**
(2000-01-01 00:00:00 UTC), stored as a [`u32`].  All transaction fields
that carry a timestamp - `Expiration`, `FinishAfter`, `CancelAfter` - use
this unit.

UNIX timestamps (seconds since 1970-01-01) are offset by exactly
**946 684 800** seconds.  Passing a UNIX timestamp directly into one of
those fields silently sets the time ~30 years in the future; these helpers
make the conversion explicit.

# Examples

```rust
use xrpl::time::{unix_to_ripple, ripple_to_unix, ripple_now};

let unix_secs: u64 = 1_000_000_000; // 2001-09-09
let ripple = unix_to_ripple(unix_secs);
assert_eq!(ripple_to_unix(ripple), unix_secs);

let now: u32 = ripple_now();
let expiry = now + 3600; // 1 hour from now in Ripple epoch
```

```rust
pub mod time { /* ... */ }
```

### Functions

#### Function `unix_to_ripple`

Converts a UNIX timestamp (seconds since 1970-01-01 UTC) to a Ripple epoch
timestamp (seconds since 2000-01-01 UTC).

# Panics

Panics if `unix_secs` is less than [`RIPPLE_EPOCH_OFFSET`] (i.e. before
2000-01-01) or if the result overflows [`u32`].

```rust
pub fn unix_to_ripple(unix_secs: u64) -> u32 { /* ... */ }
```

#### Function `ripple_to_unix`

Converts a Ripple epoch timestamp (seconds since 2000-01-01 UTC) to a UNIX
timestamp (seconds since 1970-01-01 UTC).

```rust
pub fn ripple_to_unix(ripple_secs: u32) -> u64 { /* ... */ }
```

#### Function `ripple_now`

Returns the current time as seconds since the Ripple epoch (2000-01-01 UTC).

```rust
pub fn ripple_now() -> u32 { /* ... */ }
```

### Constants and Statics

#### Constant `RIPPLE_EPOCH_OFFSET`

Seconds between the UNIX epoch (1970-01-01) and the Ripple epoch (2000-01-01).

```rust
pub const RIPPLE_EPOCH_OFFSET: u64 = 946_684_800;
```

## Module `types`

Transaction, account-object, amount, and builder types.

```rust
pub mod types { /* ... */ }
```

### Modules

## Module `account_flag`

Account flags bridging two numbering systems: `asf*` indices used in
`AccountSet` `SetFlag`/`ClearFlag` fields, and `lsf*` bitmasks returned
in the `Flags` field of `account_info`. Lives in its own module because
per-transaction flag types (e.g. `PaymentFlags`) only have one representation.

```rust
pub mod account_flag { /* ... */ }
```

### Types

#### Struct `AccountFlags`

The active account flags as returned by `account_info`.

Wraps the raw `Flags` bitmask from `AccountRoot` and provides typed access
via [`has`](Self::has). The raw value is preserved so that bits from unknown
amendments are never silently discarded.

# Examples

```rust
use xrpl::types::{AccountFlag, AccountFlags};

let flags = AccountFlags::from(0x00820000_u32); // DefaultRipple + RequireDest
assert!(flags.has(AccountFlag::DefaultRipple));
assert!(flags.has(AccountFlag::RequireDest));
assert!(!flags.has(AccountFlag::DisableMaster));
```

```rust
pub struct AccountFlags(/* private field */);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `private` | *Private field* |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: AccountFlag) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set.

- ```rust
  pub fn raw(self: Self) -> u32 { /* ... */ }
  ```
  The raw bitmask as received from the ledger.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<D: Deserializer<''de>>(d: D) -> Result<Self, <D as >::Error> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<S: Serializer>(self: &Self, s: S) -> Result<<S as >::Ok, <S as >::Error> { /* ... */ }
    ```

#### Enum `AccountFlag`

**Attributes:**

- `NonExhaustive`

An account-level flag used in [`AccountSet`](crate::types::transactions::account::AccountSet) transactions.

Each variant encodes both representations: the `asf*` integer index used in
`SetFlag`/`ClearFlag` fields (via [`asf_index`](Self::asf_index)) and the
`lsf*` bitmask used in `account_info` `Flags` fields
(via [`lsf_mask`](Self::lsf_mask)).

# Examples

```rust
use xrpl::types::AccountFlag;

assert_eq!(AccountFlag::RequireDest.asf_index(), 1);
assert_eq!(AccountFlag::RequireDest.lsf_mask(), 0x00020000);
```

```rust
pub enum AccountFlag {
    RequireDest,
    RequireAuth,
    DisallowXrp,
    DisableMaster,
    AccountTxnId,
    NoFreeze,
    GlobalFreeze,
    DefaultRipple,
    DepositAuth,
    AuthorizedNftokenMinter,
    DisallowIncomingNftokenOffer,
    DisallowIncomingCheck,
    DisallowIncomingPayChan,
    DisallowIncomingTrustline,
    AllowTrustLineClawback,
    AllowTrustLineLocking,
    Unknown(u32),
}
```

##### Variants

###### `RequireDest`

Require a destination tag on all incoming transactions.

###### `RequireAuth`

Require authorization before users can hold this account's issued tokens.

###### `DisallowXrp`

Advisory: request that senders do not send XRP to this account.

###### `DisableMaster`

Disable the master key pair; a regular key or signer list must exist first.

###### `AccountTxnId`

Track the ID of this account's most recent transaction.

###### `NoFreeze`

Permanently give up the ability to freeze individual trust lines or apply Global Freeze.

###### `GlobalFreeze`

Freeze all assets issued by this account.

###### `DefaultRipple`

Enable rippling on trust lines by default; required for token issuers.

###### `DepositAuth`

Enable Deposit Authorization; only pre-authorized senders can deposit.

###### `AuthorizedNftokenMinter`

Authorize another account to mint NFTokens on behalf of this account.

###### `DisallowIncomingNftokenOffer`

Block incoming NFTokenOffer objects directed at this account.

###### `DisallowIncomingCheck`

Block incoming Check objects directed at this account.

###### `DisallowIncomingPayChan`

Block incoming PayChannel objects directed at this account.

###### `DisallowIncomingTrustline`

Block incoming TrustLine objects directed at this account.

###### `AllowTrustLineClawback`

Allow the account to claw back tokens it has issued. Irreversible once set.

###### `AllowTrustLineLocking`

Allow trust line tokens issued by this account to be held in escrow. Irreversible once set.

###### `Unknown`

An unrecognized flag value from a protocol amendment not yet reflected in this library.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn asf_index(self: Self) -> u32 { /* ... */ }
  ```
  The `asf*` integer index used in [`AccountSet`](crate::types::transactions::account::AccountSet) `SetFlag`/`ClearFlag` fields.

- ```rust
  pub fn lsf_mask(self: Self) -> u32 { /* ... */ }
  ```
  The `lsf*` bitmask for checking this flag in `account_info` `Flags` fields.

- ```rust
  pub fn from_asf_index(v: u32) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<D: Deserializer<''de>>(d: D) -> Result<Self, <D as >::Error> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<S: Serializer>(self: &Self, s: S) -> Result<<S as >::Ok, <S as >::Error> { /* ... */ }
    ```

## Module `account_object`

Ledger-object types returned by `account_objects`.

```rust
pub mod account_object { /* ... */ }
```

### Types

#### Enum `AccountObject`

**Attributes:**

- `Other("#[serde(tag = \"LedgerEntryType\")]")`

Any ledger object that an account can own, discriminated by `LedgerEntryType`.

Returned inside the `account_objects` array of the `account_objects` RPC command.
Match on this enum to access type-specific fields without casting.

# Examples

```rust
use xrpl::types::account_object::AccountObject;
// Typically deserialized from the account_objects RPC response.
```

```rust
pub enum AccountObject {
    Bridge(Bridge),
    Check(Check),
    Credential(Credential),
    DepositPreauth(DepositPreauth),
    Did(Did),
    Escrow(Escrow),
    MPToken(MPToken),
    MPTokenIssuance(MPTokenIssuance),
    NFTokenOffer(NFTokenOffer),
    NFTokenPage(NFTokenPage),
    Offer(Offer),
    Oracle(Oracle),
    PayChannel(PayChannel),
    RippleState(RippleState),
    SignerList(SignerList),
    Ticket(Ticket),
    XChainOwnedClaimID(XChainOwnedClaimID),
    XChainOwnedCreateAccountClaimID(XChainOwnedCreateAccountClaimID),
}
```

##### Variants

###### `Bridge`

A cross-chain bridge door entry owned by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Bridge` |  |

###### `Check`

A deferred payment check that can be cashed by the destination.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Check` |  |

###### `Credential`

A verifiable credential issued to or by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Credential` |  |

###### `DepositPreauth`

A deposit pre-authorization granted by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `DepositPreauth` |  |

###### `Did`

A Decentralized Identifier (DID) document anchored to this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Did` |  |

###### `Escrow`

A time-locked or condition-locked XRP escrow.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Escrow` |  |

###### `MPToken`

A Multi-Purpose Token holding owned by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `MPToken` |  |

###### `MPTokenIssuance`

An MPT issuance created by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `MPTokenIssuance` |  |

###### `NFTokenOffer`

An offer to buy or sell an NFToken.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `NFTokenOffer` |  |

###### `NFTokenPage`

A page of NFTokens stored in this account's collection.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `NFTokenPage` |  |

###### `Offer`

A DEX offer to exchange one asset for another.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Offer` |  |

###### `Oracle`

A price oracle entry published by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Oracle` |  |

###### `PayChannel`

A payment channel funded by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `PayChannel` |  |

###### `RippleState`

A trust line (RippleState) between this account and a counterparty.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `RippleState` |  |

###### `SignerList`

A multi-signature signer list associated with this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `SignerList` |  |

###### `Ticket`

A sequence-number ticket reserved for a future transaction.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Ticket` |  |

###### `XChainOwnedClaimID`

A cross-chain claim ID owned by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `XChainOwnedClaimID` |  |

###### `XChainOwnedCreateAccountClaimID`

A cross-chain create-account claim ID owned by this account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `XChainOwnedCreateAccountClaimID` |  |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Common`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Fields present on every ledger object; flattened into each concrete type.

```rust
pub struct Common {
    pub flags: u32,
    pub index: Option<String>,
    pub owner_node: Option<String>,
    pub previous_txn_id: Option<String>,
    pub previous_txn_lgr_seq: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `flags` | `u32` | Bitfield of object-specific flags. |
| `index` | `Option<String>` | Ledger object index (hash), when included in responses. |
| `owner_node` | `Option<String>` | Index into the owner directory page that holds this object. |
| `previous_txn_id` | `Option<String>` | Hash of the transaction that most recently modified this object. |
| `previous_txn_lgr_seq` | `Option<u32>` | Ledger sequence of the transaction that most recently modified this object. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Check`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A deferred payment check that the destination can cash for up to `send_max`.

```rust
pub struct Check {
    pub account: String,
    pub destination: String,
    pub destination_node: Option<String>,
    pub destination_tag: Option<u32>,
    pub expiration: Option<u32>,
    pub invoice_id: Option<String>,
    pub send_max: serde_json::Value,
    pub sequence: u32,
    pub source_tag: Option<u32>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that created the check. |
| `destination` | `String` | r-address of the account authorized to cash the check. |
| `destination_node` | `Option<String>` | Index into the destination's owner directory. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `expiration` | `Option<u32>` | Ripple epoch time after which the check can no longer be cashed. |
| `invoice_id` | `Option<String>` | Optional 256-bit hash identifying the invoice this check is for. |
| `send_max` | `serde_json::Value` | Maximum amount the destination can receive when cashing. |
| `sequence` | `u32` | Sequence number of the CheckCreate transaction. |
| `source_tag` | `Option<u32>` | Source tag for routing within the sending account. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Credential`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A verifiable credential issued to `account` by `issuer`.

```rust
pub struct Credential {
    pub account: String,
    pub issuer: String,
    pub credential_type: String,
    pub expiration: Option<u32>,
    pub uri: Option<String>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the credential subject (holder). |
| `issuer` | `String` | r-address of the credential issuer. |
| `credential_type` | `String` | Hex-encoded credential type identifier. |
| `expiration` | `Option<u32>` | Ripple epoch time after which the credential expires. |
| `uri` | `Option<String>` | Optional URI pointing to additional credential metadata. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `DepositPreauth`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A deposit pre-authorization allowing a specific sender to make payments.

```rust
pub struct DepositPreauth {
    pub account: String,
    pub authorize: Option<String>,
    pub authorize_credentials: Option<serde_json::Value>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that granted the pre-authorization. |
| `authorize` | `Option<String>` | r-address of the account granted permission to send deposits. |
| `authorize_credentials` | `Option<serde_json::Value>` | Credential-based authorization entries (XLS-34). |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Did`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A Decentralized Identifier (DID) document anchored on the XRPL.

```rust
pub struct Did {
    pub account: String,
    pub did_document: Option<String>,
    pub data: Option<String>,
    pub uri: Option<String>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the DID subject. |
| `did_document` | `Option<String>` | Hex-encoded W3C DID document (optional, stored on-ledger). |
| `data` | `Option<String>` | Hex-encoded arbitrary data attached to the DID. |
| `uri` | `Option<String>` | URI pointing to off-ledger DID document or metadata. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Escrow`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

An XRP amount held in escrow, releasable by time or crypto-condition.

```rust
pub struct Escrow {
    pub account: String,
    pub amount: String,
    pub cancel_after: Option<u32>,
    pub condition: Option<String>,
    pub destination: String,
    pub destination_node: Option<String>,
    pub destination_tag: Option<u32>,
    pub finish_after: Option<u32>,
    pub source_tag: Option<u32>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that created the escrow. |
| `amount` | `String` | Amount of XRP (in drops) held in escrow. |
| `cancel_after` | `Option<u32>` | Ripple epoch time after which the escrow can be cancelled. |
| `condition` | `Option<String>` | PREIMAGE-SHA-256 crypto-condition that must be fulfilled to release funds. |
| `destination` | `String` | r-address of the intended recipient. |
| `destination_node` | `Option<String>` | Index into the destination's owner directory. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `finish_after` | `Option<u32>` | Ripple epoch time after which the escrow can be finished. |
| `source_tag` | `Option<u32>` | Source tag for routing within the sending account. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `MPToken`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

An MPT holding owned by an account for a specific issuance.

```rust
pub struct MPToken {
    pub account: String,
    pub mpt_issuance_id: serde_json::Value,
    pub mpt_amount: serde_json::Value,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the token holder. |
| `mpt_issuance_id` | `serde_json::Value` | 48-character hex ID of the MPT issuance. |
| `mpt_amount` | `serde_json::Value` | Current token balance held by this account. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `MPTokenIssuance`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

An MPT issuance definition created by `issuer`.

```rust
pub struct MPTokenIssuance {
    pub issuer: String,
    pub asset_scale: Option<u8>,
    pub maximum_amount: Option<String>,
    pub outstanding_amount: Option<String>,
    pub transfer_fee: Option<u16>,
    pub mpt_metadata: Option<String>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `issuer` | `String` | r-address of the account that created the issuance. |
| `asset_scale` | `Option<u8>` | Decimal precision of the token (number of digits after the decimal point). |
| `maximum_amount` | `Option<String>` | Maximum number of tokens that can ever be minted (string-encoded u64). |
| `outstanding_amount` | `Option<String>` | Total tokens currently in circulation (string-encoded u64). |
| `transfer_fee` | `Option<u16>` | Transfer fee charged on secondary transfers, in units of 1/100,000. |
| `mpt_metadata` | `Option<String>` | Hex-encoded arbitrary metadata associated with the issuance. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `NFTokenOffer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

An offer to buy or sell a specific NFToken.

```rust
pub struct NFTokenOffer {
    pub amount: serde_json::Value,
    pub destination: Option<String>,
    pub expiration: Option<u32>,
    pub nftoken_id: String,
    pub nftoken_offer_node: String,
    pub owner: String,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `serde_json::Value` | Price offered (XRP drops string or issued-currency object). |
| `destination` | `Option<String>` | If set, only this r-address may accept the offer. |
| `expiration` | `Option<u32>` | Ripple epoch time after which the offer is no longer valid. |
| `nftoken_id` | `String` | 256-bit hex identifier of the NFToken being offered. |
| `nftoken_offer_node` | `String` | Index into the NFToken offer directory. |
| `owner` | `String` | r-address of the account that created this offer. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `NFTokenPage`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A page of up to 32 NFTokens stored in an account's NFToken directory.

```rust
pub struct NFTokenPage {
    pub next_page_min: Option<String>,
    pub nftokens: serde_json::Value,
    pub previous_page_min: Option<String>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `next_page_min` | `Option<String>` | Low boundary of the next page's NFToken IDs, used for pagination. |
| `nftokens` | `serde_json::Value` | Array of NFToken objects on this page. |
| `previous_page_min` | `Option<String>` | High boundary of the previous page's NFToken IDs, used for pagination. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Offer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A DEX offer to exchange `taker_pays` for `taker_gets`.

```rust
pub struct Offer {
    pub account: String,
    pub book_directory: String,
    pub book_node: String,
    pub expiration: Option<u32>,
    pub sequence: u32,
    pub taker_pays: super::Amount,
    pub taker_gets: super::Amount,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that placed the offer. |
| `book_directory` | `String` | Hash of the order-book directory this offer belongs to. |
| `book_node` | `String` | Index of this offer within its order-book directory page. |
| `expiration` | `Option<u32>` | Ripple epoch time after which the offer is automatically removed. |
| `sequence` | `u32` | Sequence number of the OfferCreate transaction that created this offer. |
| `taker_pays` | `super::Amount` | Amount the taker must pay (what the offer creator wants to receive). |
| `taker_gets` | `super::Amount` | Amount the taker receives (what the offer creator is selling). |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Oracle`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A price oracle entry publishing one or more asset price feeds on-ledger.

```rust
pub struct Oracle {
    pub account: String,
    pub oracle_document_id: u32,
    pub asset_class: Option<String>,
    pub last_update_time: u32,
    pub price_data_series: serde_json::Value,
    pub provider: Option<String>,
    pub uri: Option<String>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that controls this oracle. |
| `oracle_document_id` | `u32` | Unique identifier for this oracle document within the account. |
| `asset_class` | `Option<String>` | Descriptive asset class (e.g. "currency", "commodity"). |
| `last_update_time` | `u32` | Ripple epoch time of the most recent price update. |
| `price_data_series` | `serde_json::Value` | Array of price data entries, each containing a base/quote asset pair and price. |
| `provider` | `Option<String>` | Human-readable name of the data provider. |
| `uri` | `Option<String>` | URI linking to additional information about this oracle. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `PayChannel`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A unidirectional payment channel funded by `account` for streaming payments.

```rust
pub struct PayChannel {
    pub account: String,
    pub amount: super::Amount,
    pub balance: super::Amount,
    pub cancel_after: Option<u32>,
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub destination_node: Option<String>,
    pub expiration: Option<u32>,
    pub public_key: String,
    pub settle_delay: u32,
    pub source_tag: Option<u32>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the channel funder (source). |
| `amount` | `super::Amount` | Total XRP allocated to this channel. |
| `balance` | `super::Amount` | XRP already delivered to the destination via claims. |
| `cancel_after` | `Option<u32>` | Ripple epoch time after which the channel can be force-closed. |
| `destination` | `String` | r-address of the payment recipient. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `destination_node` | `Option<String>` | Index into the destination's owner directory. |
| `expiration` | `Option<u32>` | Ripple epoch time after which the channel expires if not renewed. |
| `public_key` | `String` | Hex-encoded 33-byte public key used to verify off-chain payment claims. |
| `settle_delay` | `u32` | Minimum time in seconds the source must wait after requesting closure. |
| `source_tag` | `Option<u32>` | Source tag for routing within the sending account. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `RippleState`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A trust line between two accounts, tracking the issued-currency balance and limits.

The "low" side is the account whose r-address sorts lexicographically lower.

```rust
pub struct RippleState {
    pub balance: super::Amount,
    pub high_limit: super::Amount,
    pub high_node: String,
    pub high_quality_in: Option<u32>,
    pub high_quality_out: Option<u32>,
    pub lock_count: Option<u32>,
    pub locked_balance: Option<super::Amount>,
    pub low_limit: super::Amount,
    pub low_node: String,
    pub low_quality_in: Option<u32>,
    pub low_quality_out: Option<u32>,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `balance` | `super::Amount` | Current balance of the trust line (positive = low side holds, negative = high side holds). |
| `high_limit` | `super::Amount` | Maximum balance the high-side account is willing to hold. |
| `high_node` | `String` | Index into the high-side account's owner directory. |
| `high_quality_in` | `Option<u32>` | Quality applied to incoming transfers on the high side (rate in millionths). |
| `high_quality_out` | `Option<u32>` | Quality applied to outgoing transfers on the high side (rate in millionths). |
| `lock_count` | `Option<u32>` | Number of locked balance entries on this trust line. |
| `locked_balance` | `Option<super::Amount>` | Amount of balance currently locked (e.g. by escrow or AMM). |
| `low_limit` | `super::Amount` | Maximum balance the low-side account is willing to hold. |
| `low_node` | `String` | Index into the low-side account's owner directory. |
| `low_quality_in` | `Option<u32>` | Quality applied to incoming transfers on the low side (rate in millionths). |
| `low_quality_out` | `Option<u32>` | Quality applied to outgoing transfers on the low side (rate in millionths). |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `SignerList`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A multi-signature signer list defining the accounts and quorum for an account.

```rust
pub struct SignerList {
    pub signer_entries: Vec<SignerEntry>,
    pub signer_list_id: u32,
    pub signer_quorum: u32,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signer_entries` | `Vec<SignerEntry>` | Ordered list of signers and their weights. |
| `signer_list_id` | `u32` | Always `0` - reserved for future use. |
| `signer_quorum` | `u32` | Minimum total signer weight required to authorize a transaction. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `SignerEntry`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

One entry in a [`SignerList`], pairing an account with its signing weight.

```rust
pub struct SignerEntry {
    pub account: String,
    pub signer_weight: u16,
    pub wallet_locator: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the signer. |
| `signer_weight` | `u16` | Weight this signer contributes toward the quorum. |
| `wallet_locator` | `Option<String>` | Optional 256-bit locator for wallet software. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, signer_weight: u16) -> Self { /* ... */ }
  ```
  Creates a new `SignerEntry` with the given account and weight.

- ```rust
  pub fn with_wallet_locator</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, wallet_locator: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Attaches a 256-bit wallet locator.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `Ticket`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A sequence-number ticket that reserves a future transaction slot.

```rust
pub struct Ticket {
    pub account: String,
    pub ticket_sequence: u32,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that created the ticket. |
| `ticket_sequence` | `u32` | The sequence number set aside for the ticket. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `Bridge`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A cross-chain bridge door object managed by `account`.

```rust
pub struct Bridge {
    pub account: String,
    pub min_account_create_amount: Option<String>,
    pub signature_reward: String,
    pub xchain_account_claim_count: String,
    pub xchain_account_create_count: String,
    pub xchain_bridge: super::XChainBridge,
    pub xchain_claim_id: String,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the bridge door account on this chain. |
| `min_account_create_amount` | `Option<String>` | Minimum XRP amount (drops) required to create an account via the bridge. |
| `signature_reward` | `String` | XRP reward paid to attestation signers per cross-chain transfer. |
| `xchain_account_claim_count` | `String` | Running count of cross-chain claim transactions processed. |
| `xchain_account_create_count` | `String` | Running count of cross-chain account-create transactions processed. |
| `xchain_bridge` | `super::XChainBridge` | Bridge definition identifying both door accounts and assets. |
| `xchain_claim_id` | `String` | The next available cross-chain claim ID. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `XChainOwnedClaimID`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A cross-chain claim ID that collects attestations for a pending bridge transfer.

```rust
pub struct XChainOwnedClaimID {
    pub account: String,
    pub other_chain_source: String,
    pub signature_reward: String,
    pub xchain_bridge: super::XChainBridge,
    pub xchain_claim_id: String,
    pub xchain_claim_attestations: serde_json::Value,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that created this claim ID. |
| `other_chain_source` | `String` | r-address of the source account on the other chain. |
| `signature_reward` | `String` | XRP (drops) paid to attestation signers for this transfer. |
| `xchain_bridge` | `super::XChainBridge` | Bridge this claim ID belongs to. |
| `xchain_claim_id` | `String` | Unique numeric identifier for this cross-chain transfer. |
| `xchain_claim_attestations` | `serde_json::Value` | Collected attestations from bridge signers. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

#### Struct `XChainOwnedCreateAccountClaimID`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

A cross-chain claim ID for a bridge transfer that creates a new account on the destination chain.

```rust
pub struct XChainOwnedCreateAccountClaimID {
    pub account: String,
    pub xchain_account_create_count: String,
    pub xchain_bridge: super::XChainBridge,
    pub xchain_create_account_attestations: serde_json::Value,
    pub common: Common,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that initiated this account-create transfer. |
| `xchain_account_create_count` | `String` | Sequence counter matching the bridge's `XChainAccountCreateCount` when the transfer was initiated. |
| `xchain_bridge` | `super::XChainBridge` | Bridge this claim ID belongs to. |
| `xchain_create_account_attestations` | `serde_json::Value` | Collected attestations from bridge signers for the account-create transfer. |
| `common` | `Common` | Shared ledger-object metadata (flags, index, previous transaction reference). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

## Module `amm`

AMM pool types returned by `amm_info`.

```rust
pub mod amm { /* ... */ }
```

### Types

#### Struct `VoteEntry`

One LP's vote for the AMM trading fee.

# Examples

```rust
use xrpl::types::amm::VoteEntry;
// Returned as part of VoteSlots in an Amm ledger object.
```

```rust
pub struct VoteEntry {
    pub account: String,
    pub trading_fee: u16,
    pub vote_weight: u64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the LP casting the vote. |
| `trading_fee` | `u16` | Proposed trading fee in units of 1/100,000 (e.g. 500 = 0.5%). |
| `vote_weight` | `u64` | Weight of this vote, proportional to the LP's token share. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `VoteSlotWrapper`

Wire-format wrapper that nests a [`VoteEntry`] under the `VoteEntry` key.

```rust
pub struct VoteSlotWrapper {
    pub vote_entry: VoteEntry,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `vote_entry` | `VoteEntry` | The contained vote entry. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `AuthAccount`

An account authorized to trade at a discounted fee during an auction slot.

```rust
pub struct AuthAccount {
    pub account: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the authorized account. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new `AuthAccount` for the given r-address.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `AuthAccountWrapper`

Wire-format wrapper that nests an [`AuthAccount`] under the `AuthAccount` key.

```rust
pub struct AuthAccountWrapper {
    pub auth_account: AuthAccount,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `auth_account` | `AuthAccount` | The contained authorized account. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Type Alias `AuthAccountEntry`

Alias kept for compatibility - the inner type of an auth-accounts list entry.

```rust
pub type AuthAccountEntry = AuthAccount;
```

#### Struct `AuctionSlot`

The currently active auction slot of an AMM pool.

The auction-slot holder pays a discounted trading fee for the slot duration.

# Examples

```rust
use xrpl::types::amm::AuctionSlot;
// Returned as part of an Amm ledger object when a slot is active.
```

```rust
pub struct AuctionSlot {
    pub account: String,
    pub auth_accounts: Option<Vec<AuthAccountEntry>>,
    pub discounted_fee: Option<u32>,
    pub price: super::Amount,
    pub expiration: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account that won the auction. |
| `auth_accounts` | `Option<Vec<AuthAccountEntry>>` | Additional accounts granted the discounted fee alongside the slot holder. |
| `discounted_fee` | `Option<u32>` | Trading fee charged to the slot holder, in units of 1/100,000. |
| `price` | `super::Amount` | Amount of LP tokens paid for the slot. |
| `expiration` | `u32` | Ledger sequence at which the slot expires. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `Amm`

Ledger object representing an Automated Market Maker (AMM) pool.

Returned by the `amm_info` RPC command. Holds the full on-chain state of a
two-asset constant-product pool, including the current LP-token supply,
trading fee, and any active auction slot.

# Examples

```rust
use xrpl::types::amm::Amm;
// Typically obtained via the amm_info WebSocket command response.
```

```rust
pub struct Amm {
    pub account: String,
    pub asset: super::Asset,
    pub asset2: super::Asset,
    pub auction_slot: Option<AuctionSlot>,
    pub flags: Option<u32>,
    pub lp_token_balance: super::Amount,
    pub ledger_entry_type: Option<String>,
    pub owner_node: Option<String>,
    pub previous_txn_id: Option<String>,
    pub previous_txn_lgr_seq: Option<u32>,
    pub trading_fee: u16,
    pub vote_slots: Option<Vec<VoteSlotWrapper>>,
    pub index: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | Special AMM account that holds the pooled assets. |
| `asset` | `super::Asset` | First asset in the pool. |
| `asset2` | `super::Asset` | Second asset in the pool. |
| `auction_slot` | `Option<AuctionSlot>` | Active auction slot, if any. |
| `flags` | `Option<u32>` | Bitfield of AMM flags. |
| `lp_token_balance` | `super::Amount` | Total outstanding LP token balance for this pool. |
| `ledger_entry_type` | `Option<String>` | Ledger entry type discriminator (always `"AMM"`). |
| `owner_node` | `Option<String>` | Index into the owner directory of the AMM account. |
| `previous_txn_id` | `Option<String>` | Hash of the transaction that most recently modified this object. |
| `previous_txn_lgr_seq` | `Option<u32>` | Ledger sequence of the transaction that most recently modified this object. |
| `trading_fee` | `u16` | Current pool trading fee in units of 1/100,000 (e.g. 500 = 0.5%). |
| `vote_slots` | `Option<Vec<VoteSlotWrapper>>` | Active fee-vote slots cast by LPs. |
| `index` | `Option<String>` | Ledger object index (hash). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `amount`

**Attributes:**

- `Other("#[attr = MacroUse {arguments: UseAll}]")`

`Amount` enum representing XRP drops, issued-currency amounts, or MPT amounts.

```rust
pub mod amount { /* ... */ }
```

### Types

#### Enum `Amount`

**Attributes:**

- `Other("#[serde(untagged)]")`

Represents an amount of currency on the XRPL: XRP, tokens, or MPTs.

# Construction

Three layers, pick the one that fits the situation:

1. **Literals in source code** - use the macros [`xrp!`], [`drops!`],
   [`issued!`], [`mpt!`]. They panic on invalid input, which is fine for
   constants the author controls.
2. **Runtime / untrusted input** - use the fallible constructors
   [`Amount::xrp`], [`Amount::drops`], [`Amount::issued_currency`],
   [`Amount::mpt`]. They return a [`Result`] and validate every field.
3. **Your own domain type** - implement [`From<MyType> for Amount`] once
   and pass `MyType` directly to any builder method (they all accept
   `impl Into<Amount>`). Implement [`TryFrom<Amount> for MyType`] to
   recover your type from ledger responses.

[`xrp!`]: crate::xrp
[`drops!`]: crate::drops
[`issued!`]: crate::issued
[`mpt!`]: crate::mpt
[`From<MyType> for Amount`]: From
[`TryFrom<Amount> for MyType`]: TryFrom

# Examples

Create an XRP amount (1.5 XRP):
```rust
use xrpl::types::Amount;
let amount = Amount::xrp("1.5").unwrap();
```

Create a token amount (100 USD issued by rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh):
```rust
use xrpl::types::Amount;
let amount = Amount::issued_currency("100", "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
```

Create an MPT amount:
```rust
use xrpl::types::Amount;
let amount = Amount::mpt("1000000", "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
```

Same amounts using the literal macros:
```rust
use xrpl::{xrp, drops, issued, mpt};
let xrp_amount = xrp!(1.5);
let same_xrp = drops!(1_500_000);
let usd = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
let token = mpt!(1_000_000, "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47");
```

Interop with your own domain type by implementing [`From`] and [`TryFrom`].
This lets your type be passed wherever an `impl Into<Amount>` is expected
(such as builder methods), and lets you recover your type from an `Amount`
returned by the ledger:

```rust
use xrpl::types::Amount;

/// A domain type holding USD cents as an unsigned integer.
struct Usd { cents: u64 }

const USD_ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

impl From<Usd> for Amount {
    fn from(usd: Usd) -> Self {
        let dollars = format!("{}.{:02}", usd.cents / 100, usd.cents % 100);
        Amount::issued_currency(dollars, "USD", USD_ISSUER).unwrap()
    }
}

impl TryFrom<Amount> for Usd {
    type Error = &'static str;
    fn try_from(a: Amount) -> Result<Self, Self::Error> {
        match a {
            Amount::IssuedCurrency { value, currency, issuer }
                if currency == "USD" && issuer == USD_ISSUER =>
            {
                let dollars: f64 = value.parse().map_err(|_| "bad value")?;
                Ok(Usd { cents: (dollars * 100.0).round() as u64 })
            }
            _ => Err("not a USD amount from the expected issuer"),
        }
    }
}

let amount: Amount = Usd { cents: 12_345 }.into();
let back: Usd = amount.try_into().unwrap();
assert_eq!(back.cents, 12_345);
```

```rust
pub enum Amount {
    Xrpl(String),
    IssuedCurrency {
        value: String,
        currency: String,
        issuer: String,
    },
    Mpt {
        value: String,
        mpt_issuance_id: String,
    },
}
```

##### Variants

###### `Xrpl`

XRP amount in drops (string format)

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `IssuedCurrency`

Token amount (issued currency)

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `value` | `String` | Numeric value as a string (supports scientific notation for tokens). |
| `currency` | `String` | 3-character standard currency code or 40-hex non-standard code. |
| `issuer` | `String` | r-address of the token issuer. |

###### `Mpt`

MPT amount (Multi-Purpose Token)

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `value` | `String` | Token quantity as a decimal string (positive integer for MPTs). |
| `mpt_issuance_id` | `String` | 48-character hex MPT issuance ID. |

##### Implementations

###### Methods

- ```rust
  pub fn xrp<T: AsRef<str>>(value: T) -> Result<Self, ValidationError> { /* ... */ }
  ```
  Create XRP amount from XRP value (converts to drops).

- ```rust
  pub fn drops<T: AsRef<str>>(value: T) -> Result<Self, ValidationError> { /* ... */ }
  ```
  Create XRP amount from drops (string format)

- ```rust
  pub fn issued_currency<V, C, I>(value: V, currency: C, issuer: I) -> Result<Self, ValidationError>
where
    V: AsRef<str>,
    C: AsRef<str>,
    I: AsRef<str> { /* ... */ }
  ```
  Create issued currency (token) amount

- ```rust
  pub fn mpt<V, I>(value: V, mpt_issuance_id: I) -> Result<Self, ValidationError>
where
    V: AsRef<str>,
    I: AsRef<str> { /* ... */ }
  ```
  Create MPT (Multi-Purpose Token) amount

- ```rust
  pub fn value(self: &Self) -> &str { /* ... */ }
  ```
  Returns the amount value as a string slice.

- ```rust
  pub fn currency(self: &Self) -> &str { /* ... */ }
  ```
  Returns the currency code. Returns `"XRP"` for XRP amounts and an empty string for MPTs.

- ```rust
  pub fn issuer(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Returns the issuer address if this is an issued currency amount, otherwise `None`.

- ```rust
  pub fn mpt_issuance_id(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Returns the MPT issuance ID if this is an MPT amount, otherwise `None`.

- ```rust
  pub fn to_drops(self: &Self) -> Option<u64> { /* ... */ }
  ```
  Returns the amount in drops as a `u64`. Returns `None` for issued currency and MPT amounts.

- ```rust
  pub fn as_drops(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Returns the raw drops string for XRP amounts. Returns `None` for issued currency and MPT amounts.

- ```rust
  pub fn to_decimal(self: &Self) -> Option<f64> { /* ... */ }
  ```
  Returns the amount as a decimal. For XRP, converts drops to XRP units. Returns `None` for MPTs.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut fmt::Formatter<''_>) -> fmt::Result { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `asset`

`Asset` enum identifying a tradable asset without a concrete quantity.

```rust
pub mod asset { /* ... */ }
```

### Types

#### Enum `Asset`

**Attributes:**

- `Other("#[serde(untagged)]")`

Identifies a tradable asset on the XRPL without specifying an amount.

Use an `Asset` to describe a pool side, order book entry, or trust-line
target where only the asset identity matters, not a concrete quantity.

# Construction

1. **Constructors** - [`Asset::xrp`] is infallible; [`Asset::token`] and
   [`Asset::mpt`] validate their inputs and return a [`Result`].
2. **From an [`Amount`]** - drop the value with
   [`Asset::try_from(amount)`][`TryFrom<Amount>`] or
   [`Asset::try_from(&amount)`][`TryFrom<&Amount>`].
3. **Your own domain type** - implement [`From<MyType> for Asset`] once and
   pass `MyType` directly to any builder method (they all accept
   `impl Into<Asset>`).

To go the other direction - pair an `Asset` with a value to produce an
[`Amount`] - use [`Asset::amount_with`].

[`TryFrom<Amount>`]: Asset#impl-TryFrom<Amount>-for-Asset
[`TryFrom<&Amount>`]: Asset#impl-TryFrom<%26Amount>-for-Asset
[`From<MyType> for Asset`]: From

# Examples

Constructors:
```rust
use xrpl::types::Asset;

let xrp   = Asset::xrp();
let usd   = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
let mpt   = Asset::mpt("0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47").unwrap();
```

Round-trip with [`Amount`]:
```rust
use xrpl::types::{Amount, Asset};

let asset = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap();
let amount = asset.amount_with("100").unwrap();
let stripped = Asset::try_from(&amount).unwrap();
assert_eq!(stripped, asset);
```

Interop with your own domain type:
```rust
use xrpl::types::Asset;

enum Currency { Native, Usd, Eur }

const USD_ISSUER: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
const EUR_ISSUER: &str = "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe";

impl From<Currency> for Asset {
    fn from(c: Currency) -> Self {
        match c {
            Currency::Native => Asset::xrp(),
            Currency::Usd => Asset::token("USD", USD_ISSUER).unwrap(),
            Currency::Eur => Asset::token("EUR", EUR_ISSUER).unwrap(),
        }
    }
}

// Now `Currency::Usd` can be passed wherever `impl Into<Asset>` is expected,
// including AMM builder methods.
let asset: Asset = Currency::Usd.into();
```

```rust
pub enum Asset {
    Token {
        currency: String,
        issuer: String,
    },
    Mpt {
        mpt_issuance_id: String,
    },
    Xrp {
        currency: String,
    },
}
```

##### Variants

###### `Token`

Issued-currency token identified by currency code and issuer address.

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `currency` | `String` | 3-character standard currency code or 40-hex non-standard code. |
| `issuer` | `String` | r-address of the token issuer. |

###### `Mpt`

Multi-Purpose Token identified by its 48-character hex issuance ID.

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `mpt_issuance_id` | `String` | 48-character hex MPT issuance ID. |

###### `Xrp`

Native XRP asset (serialized with `"currency": "XRP"`).

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `currency` | `String` | Always `"XRP"`. |

##### Implementations

###### Methods

- ```rust
  pub fn xrp() -> Self { /* ... */ }
  ```
  Returns an XRP asset descriptor.

- ```rust
  pub fn token<C, I>(currency: C, issuer: I) -> Result<Self, ValidationError>
where
    C: AsRef<str>,
    I: AsRef<str> { /* ... */ }
  ```
  Returns an issued-currency token asset descriptor, validating the currency code and issuer address.

- ```rust
  pub fn mpt<I>(mpt_issuance_id: I) -> Result<Self, ValidationError>
where
    I: AsRef<str> { /* ... */ }
  ```
  Returns an MPT asset descriptor, validating the 48-character hex issuance ID.

- ```rust
  pub fn currency(self: &Self) -> &str { /* ... */ }
  ```
  Returns the currency code. Returns `"XRP"` for XRP assets and an empty string for MPTs.

- ```rust
  pub fn issuer(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Returns the issuer address if this is a token asset, otherwise `None`.

- ```rust
  pub fn mpt_issuance_id(self: &Self) -> Option<&str> { /* ... */ }
  ```
  Returns the MPT issuance ID if this is an MPT asset, otherwise `None`.

- ```rust
  pub fn amount_with<V: AsRef<str>>(self: &Self, value: V) -> Result<Amount, ValidationError> { /* ... */ }
  ```
  Produce an [`Amount`] by pairing this asset with a value.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut fmt::Formatter<''_>) -> fmt::Result { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `builders`

Transaction builder types for all XRPL transaction types.

```rust
pub mod builders { /* ... */ }
```

### Re-exports

#### Re-export `AccountDeleteBuilder`

```rust
pub use account_delete::AccountDeleteBuilder;
```

#### Re-export `AccountSetBuilder`

```rust
pub use account_set::AccountSetBuilder;
```

#### Re-export `AMMBidBuilder`

```rust
pub use amm_bid::AMMBidBuilder;
```

#### Re-export `AMMCreateBuilder`

```rust
pub use amm_create::AMMCreateBuilder;
```

#### Re-export `AMMDepositBuilder`

```rust
pub use amm_deposit::AMMDepositBuilder;
```

#### Re-export `AMMWithdrawBuilder`

```rust
pub use amm_withdraw::AMMWithdrawBuilder;
```

#### Re-export `AMMVoteBuilder`

```rust
pub use amm_vote::AMMVoteBuilder;
```

#### Re-export `AMMDeleteBuilder`

```rust
pub use amm_delete::AMMDeleteBuilder;
```

#### Re-export `AMMClawbackBuilder`

```rust
pub use amm_clawback::AMMClawbackBuilder;
```

#### Re-export `CheckCancelBuilder`

```rust
pub use check_cancel::CheckCancelBuilder;
```

#### Re-export `CheckCashBuilder`

```rust
pub use check_cash::CheckCashBuilder;
```

#### Re-export `CheckCreateBuilder`

```rust
pub use check_create::CheckCreateBuilder;
```

#### Re-export `ClawbackBuilder`

```rust
pub use clawback::ClawbackBuilder;
```

#### Re-export `CredentialAcceptBuilder`

```rust
pub use credential_accept::CredentialAcceptBuilder;
```

#### Re-export `CredentialCreateBuilder`

```rust
pub use credential_create::CredentialCreateBuilder;
```

#### Re-export `CredentialDeleteBuilder`

```rust
pub use credential_delete::CredentialDeleteBuilder;
```

#### Re-export `DepositPreauthBuilder`

```rust
pub use deposit_preauth::DepositPreauthBuilder;
```

#### Re-export `DIDDeleteBuilder`

```rust
pub use did_delete::DIDDeleteBuilder;
```

#### Re-export `DIDSetBuilder`

```rust
pub use did_set::DIDSetBuilder;
```

#### Re-export `EscrowCancelBuilder`

```rust
pub use escrow_cancel::EscrowCancelBuilder;
```

#### Re-export `EscrowCreateBuilder`

```rust
pub use escrow_create::EscrowCreateBuilder;
```

#### Re-export `EscrowFinishBuilder`

```rust
pub use escrow_finish::EscrowFinishBuilder;
```

#### Re-export `MPTokenAuthorizeBuilder`

```rust
pub use mpt_authorize::MPTokenAuthorizeBuilder;
```

#### Re-export `MPTokenIssuanceCreateBuilder`

```rust
pub use mpt_issuance_create::MPTokenIssuanceCreateBuilder;
```

#### Re-export `MPTokenIssuanceDestroyBuilder`

```rust
pub use mpt_issuance_destroy::MPTokenIssuanceDestroyBuilder;
```

#### Re-export `MPTokenIssuanceSetBuilder`

```rust
pub use mpt_issuance_set::MPTokenIssuanceSetBuilder;
```

#### Re-export `NFTokenAcceptOfferBuilder`

```rust
pub use nftoken_accept_offer::NFTokenAcceptOfferBuilder;
```

#### Re-export `NFTokenBurnBuilder`

```rust
pub use nftoken_burn::NFTokenBurnBuilder;
```

#### Re-export `NFTokenCancelOfferBuilder`

```rust
pub use nftoken_cancel_offer::NFTokenCancelOfferBuilder;
```

#### Re-export `NFTokenCreateOfferBuilder`

```rust
pub use nftoken_create_offer::NFTokenCreateOfferBuilder;
```

#### Re-export `NFTokenMintBuilder`

```rust
pub use nftoken_mint::NFTokenMintBuilder;
```

#### Re-export `OfferCancelBuilder`

```rust
pub use offer_cancel::OfferCancelBuilder;
```

#### Re-export `OfferCreateBuilder`

```rust
pub use offer_create::OfferCreateBuilder;
```

#### Re-export `OracleDeleteBuilder`

```rust
pub use oracle_delete::OracleDeleteBuilder;
```

#### Re-export `OracleSetBuilder`

```rust
pub use oracle_set::OracleSetBuilder;
```

#### Re-export `PaymentBuilder`

```rust
pub use payment::PaymentBuilder;
```

#### Re-export `PaymentChannelClaimBuilder`

```rust
pub use payment_channel_claim::PaymentChannelClaimBuilder;
```

#### Re-export `PaymentChannelCreateBuilder`

```rust
pub use payment_channel_create::PaymentChannelCreateBuilder;
```

#### Re-export `PaymentChannelFundBuilder`

```rust
pub use payment_channel_fund::PaymentChannelFundBuilder;
```

#### Re-export `SetRegularKeyBuilder`

```rust
pub use set_regular_key::SetRegularKeyBuilder;
```

#### Re-export `SignerListSetBuilder`

```rust
pub use signer_list_set::SignerListSetBuilder;
```

#### Re-export `SubmitMultisignedRequestBuilder`

```rust
pub use submit::SubmitMultisignedRequestBuilder;
```

#### Re-export `SubmitRequestBuilder`

```rust
pub use submit::SubmitRequestBuilder;
```

#### Re-export `TicketCreateBuilder`

```rust
pub use ticket_create::TicketCreateBuilder;
```

#### Re-export `TrustSetBuilder`

```rust
pub use trust_set::TrustSetBuilder;
```

#### Re-export `XChainAccountCreateCommitBuilder`

```rust
pub use xchain_account_create_commit::XChainAccountCreateCommitBuilder;
```

#### Re-export `XChainAddAccountCreateAttestationBuilder`

```rust
pub use xchain_add_account_create_attestation::XChainAddAccountCreateAttestationBuilder;
```

#### Re-export `XChainAddClaimAttestationBuilder`

```rust
pub use xchain_add_claim_attestation::XChainAddClaimAttestationBuilder;
```

#### Re-export `XChainClaimBuilder`

```rust
pub use xchain_claim::XChainClaimBuilder;
```

#### Re-export `XChainCommitBuilder`

```rust
pub use xchain_commit::XChainCommitBuilder;
```

#### Re-export `XChainCreateBridgeBuilder`

```rust
pub use xchain_create_bridge::XChainCreateBridgeBuilder;
```

#### Re-export `XChainCreateClaimIDBuilder`

```rust
pub use xchain_create_claim_id::XChainCreateClaimIDBuilder;
```

#### Re-export `XChainModifyBridgeBuilder`

```rust
pub use xchain_modify_bridge::XChainModifyBridgeBuilder;
```

#### Re-export `common::*`

```rust
pub use common::*;
```

## Module `transaction_meta`

Transaction metadata and delivered-amount types.

```rust
pub mod transaction_meta { /* ... */ }
```

### Types

#### Struct `TransactionMeta`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Execution metadata attached to every validated transaction.

Use [`delivered_amount`](Self::delivered_amount) instead of the transaction's
`Amount` field when crediting received payments - it reflects the actual amount
delivered and guards against partial-payment attacks. See [`HasTransactionMeta`]
for the possible states.

```rust
pub struct TransactionMeta {
    pub affected_nodes: Vec<serde_json::Value>,
    pub transaction_index: u32,
    pub transaction_result: String,
    pub delivered_amount: Option<super::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `affected_nodes` | `Vec<serde_json::Value>` | Ledger objects created, modified, or deleted by this transaction. |
| `transaction_index` | `u32` | Position of this transaction within the ledger (zero-based). |
| `transaction_result` | `String` | Final transaction result code (e.g. `"tesSUCCESS"`). |
| `delivered_amount` | `Option<super::Amount>` | Actual amount delivered to the destination.<br><br>Present only for Payment transactions. `None` for non-payment transactions<br>and for pre-2014 partial payments where the amount cannot be recovered.<br>Always use this instead of the transaction's `Amount` field. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

### Traits

#### Trait `HasTransactionMeta`

Implemented by any type that carries [`TransactionMeta`].

Provides [`delivered_amount`](Self::delivered_amount) as a single, safe call
for reading the actual amount received by a payment - regardless of whether
the data comes from an `account_tx` response, a `tx` response, or a
subscription message.

# Examples

```rust
use xrpl::types::HasTransactionMeta;

fn print_delivered(tx: &impl HasTransactionMeta) {
    match tx.delivered_amount() {
        Some(amount) => println!("Delivered: {amount}"),
        None => println!("Not a payment transaction"),
    }
}
```

```rust
pub trait HasTransactionMeta {
    /* Associated items */
}
```

##### Required Items

###### Required Methods

- `transaction_meta`: Returns the transaction metadata, if present.

##### Provided Methods

- ```rust
  fn delivered_amount(self: &Self) -> Option<&Amount> { /* ... */ }
  ```
  Returns the actual amount delivered to the destination.

##### Implementations

This trait is implemented for the following types:

- `AccountTransaction`
- `TxResponse`
- `AccountTransactionMessage`

## Module `transactions`

Transaction type definitions for all XRPL transaction kinds.

```rust
pub mod transactions { /* ... */ }
```

### Modules

## Module `account`

Account management transaction types (AccountSet, AccountDelete, etc.).

```rust
pub mod account { /* ... */ }
```

### Types

#### Struct `AccountDelete`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Removes a funded account from the ledger, returning its remaining XRP to `Destination`.

The account's sequence number must be at least 256 ahead of its current ledger
sequence, and the transaction costs 2 XRP in addition to the normal fee.

```rust
use xrpl::types::transactions::account::AccountDelete;
let tx = AccountDelete {
    destination: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
    destination_tag: Some(12345),
    credential_ids: None,
};
```

```rust
pub struct AccountDelete {
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub credential_ids: Option<Vec<String>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `destination` | `String` | Account that receives the remaining XRP balance. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `credential_ids` | `Option<Vec<String>>` | Credential IDs required to pass deposit authorization. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AccountSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Modifies account flags and optional properties such as domain, email hash, and transfer rate.

```rust
use xrpl::types::transactions::account::AccountSet;
use xrpl::types::AccountFlag;
let tx = AccountSet {
    set_flag: Some(AccountFlag::RequireDest),
    domain: Some("6578616d706c652e636f6d".to_string()),
    clear_flag: None,
    email_hash: None,
    message_key: None,
    transfer_rate: None,
    tick_size: None,
    nftoken_minter: None,
};
```

```rust
pub struct AccountSet {
    pub clear_flag: Option<crate::types::AccountFlag>,
    pub domain: Option<String>,
    pub email_hash: Option<String>,
    pub message_key: Option<String>,
    pub set_flag: Option<crate::types::AccountFlag>,
    pub transfer_rate: Option<u32>,
    pub tick_size: Option<u32>,
    pub nftoken_minter: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `clear_flag` | `Option<crate::types::AccountFlag>` | Account flag to disable. |
| `domain` | `Option<String>` | Hex-encoded domain name associated with the account. |
| `email_hash` | `Option<String>` | MD5 hash of an email address for Gravatar lookup. |
| `message_key` | `Option<String>` | Hex-encoded public key for encrypted messaging. |
| `set_flag` | `Option<crate::types::AccountFlag>` | Account flag to enable. |
| `transfer_rate` | `Option<u32>` | Fee charged when users receive the issuer's tokens (in billionths, e.g. 1_005_000_000 = 0.5%). |
| `tick_size` | `Option<u32>` | Minimum quote increment for offers on this account's issued currency (3-15, or 0 to disable). |
| `nftoken_minter` | `Option<String>` | Account authorized to mint NFTokens on behalf of this account. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `DepositPreauth`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Pre-authorizes or revokes a specific account's permission to send payments to this account.

Used when `DepositAuth` is enabled to explicitly whitelist senders without requiring
the sender to go through a separate authorization flow.

```rust
use xrpl::types::transactions::account::DepositPreauth;
let tx = DepositPreauth {
    authorize: Some("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string()),
    unauthorize: None,
};
```

```rust
pub struct DepositPreauth {
    pub authorize: Option<String>,
    pub unauthorize: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `authorize` | `Option<String>` | Account to grant deposit authorization to. |
| `unauthorize` | `Option<String>` | Account whose deposit authorization is revoked. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `SetRegularKey`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Assigns or removes an alternate signing key pair for the account.

After setting a regular key, the account can be signed with either the master key
or the regular key. The master key can subsequently be disabled to improve security.

```rust
use xrpl::types::transactions::account::SetRegularKey;
let tx = SetRegularKey {
    regular_key: Some("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string()),
};
```

```rust
pub struct SetRegularKey {
    pub regular_key: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `regular_key` | `Option<String>` | The alternate signing key to assign; omit to remove the current regular key. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `SignerListSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Defines or replaces the multi-signature signer list and quorum for an account.

Submit with an empty `signer_entries` to delete the signer list and revert to
single-key signing.

```rust
use xrpl::types::transactions::account::SignerListSet;
let tx = SignerListSet {
    signer_quorum: 2,
    signer_entries: None, // populated via builder
};
```

```rust
pub struct SignerListSet {
    pub signer_quorum: u32,
    pub signer_entries: Option<Vec<crate::types::SignerEntryWrapper>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signer_quorum` | `u32` | Minimum cumulative weight required to authorize a transaction. |
| `signer_entries` | `Option<Vec<crate::types::SignerEntryWrapper>>` | List of signer accounts and their individual weights. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `TicketCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Reserves one or more sequence-number slots (tickets) for out-of-order transaction submission.

Tickets allow sending transactions in an arbitrary order without being blocked by gaps
in the sequence number, which is useful for multi-signing workflows or parallel submissions.

```rust
use xrpl::types::transactions::account::TicketCreate;
let tx = TicketCreate { ticket_count: 5 };
```

```rust
pub struct TicketCreate {
    pub ticket_count: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ticket_count` | `u32` | Number of tickets to reserve (1-250 per transaction). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `amm`

AMM transaction types (AMMCreate, AMMDeposit, AMMWithdraw, etc.).

```rust
pub mod amm { /* ... */ }
```

### Types

#### Struct `AMMDepositFlags`

Transaction flags for [`AMMDeposit`].

Exactly one mode flag must be set. Combine with common flags using `|`.

```rust
use xrpl::types::AMMDepositFlags as Flags;

let flags = Flags::SINGLE_ASSET; // deposit one asset, receive LP tokens
```

```rust
pub struct AMMDepositFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `AMMWithdrawFlags`

Transaction flags for [`AMMWithdraw`].

Exactly one mode flag must be set. Combine with common flags using `|`.

```rust
use xrpl::types::AMMWithdrawFlags as Flags;

let flags = Flags::LP_TOKEN; // redeem LP tokens for both assets
```

```rust
pub struct AMMWithdrawFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `AMMBid`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Bids on the AMM auction slot to receive a discounted trading fee for a limited time.

The winning bidder pays LP tokens and can authorize up to four additional accounts
to also receive the discounted fee.

```rust
use xrpl::types::{Asset, transactions::amm::AMMBid};
let tx = AMMBid {
    asset: Asset::xrp(),
    asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    bid_min: None,
    bid_max: None,
    auth_accounts: None,
};
```

```rust
pub struct AMMBid {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
    pub bid_min: Option<crate::types::Amount>,
    pub bid_max: Option<crate::types::Amount>,
    pub auth_accounts: Option<Vec<crate::types::AuthAccountWrapper>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset of the AMM pool. |
| `asset2` | `crate::types::Asset` | Second asset of the AMM pool. |
| `bid_min` | `Option<crate::types::Amount>` | Minimum LP token amount the bidder is willing to pay. |
| `bid_max` | `Option<crate::types::Amount>` | Maximum LP token amount the bidder is willing to pay. |
| `auth_accounts` | `Option<Vec<crate::types::AuthAccountWrapper>>` | Accounts that also receive the discounted fee while the slot is held (up to 4). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMClawback`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Issuer clawback of tokens held inside an AMM pool.

Available only when the token issuance has clawback enabled. Removes the specified
holder's share of the issuer's token from the AMM pool.

```rust
use xrpl::types::{Amount, Asset, transactions::amm::AMMClawback};
let tx = AMMClawback {
    asset: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    asset2: Asset::xrp(),
    amount: None, // claw back all if omitted
    holder: "rHolderAccount".to_string(),
};
```

```rust
pub struct AMMClawback {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
    pub amount: Option<crate::types::Amount>,
    pub holder: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | The issuer's token asset in the pool. |
| `asset2` | `crate::types::Asset` | The paired asset in the pool. |
| `amount` | `Option<crate::types::Amount>` | Maximum amount to claw back; omit to claw back the full balance. |
| `holder` | `String` | The account holding the tokens to be clawed back. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Initializes a new Automated Market Maker (AMM) pool with two assets and a trading fee.

The submitting account provides the initial liquidity for both assets and receives
LP tokens representing its share of the pool.

```rust
use xrpl::types::{Amount, transactions::amm::AMMCreate};
let tx = AMMCreate {
    amount: Amount::Xrpl("50000000".to_string()),
    amount2: Amount::IssuedCurrency {
        value: "500".to_string(),
        currency: "USD".to_string(),
        issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
    },
    trading_fee: 500, // 0.5%
};
```

```rust
pub struct AMMCreate {
    pub amount: crate::types::Amount,
    pub amount2: crate::types::Amount,
    pub trading_fee: u16,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Initial deposit of the first asset. |
| `amount2` | `crate::types::Amount` | Initial deposit of the second asset. |
| `trading_fee` | `u16` | Trading fee in units of 1/100,000 of a percent (0-1000, i.e. 0%-1%). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMDelete`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Removes an AMM pool that has been reduced to an empty or dust state.

Any account can submit this transaction to clean up a pool with no remaining
liquidity and claim the reserve that was locked by the pool object.

```rust
use xrpl::types::{Asset, transactions::amm::AMMDelete};
let tx = AMMDelete {
    asset: Asset::xrp(),
    asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
};
```

```rust
pub struct AMMDelete {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset of the pool to remove. |
| `asset2` | `crate::types::Asset` | Second asset of the pool to remove. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMDeposit`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Adds liquidity to an existing AMM pool and receives LP tokens in return.

Supports several deposit modes selected by combining the optional fields and
the transaction flags (e.g. single-asset, double-asset, or LP-token-targeted).

```rust
use xrpl::types::{Amount, Asset, transactions::amm::AMMDeposit};
let tx = AMMDeposit {
    asset: Asset::xrp(),
    asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    amount: Some(Amount::Xrpl("10000000".to_string())),
    amount2: None,
    e_price: None,
    lp_token_out: None,
    trading_fee: None,
};
```

```rust
pub struct AMMDeposit {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
    pub amount: Option<crate::types::Amount>,
    pub amount2: Option<crate::types::Amount>,
    pub e_price: Option<crate::types::Amount>,
    pub lp_token_out: Option<crate::types::Amount>,
    pub trading_fee: Option<u16>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset of the pool. |
| `asset2` | `crate::types::Asset` | Second asset of the pool. |
| `amount` | `Option<crate::types::Amount>` | Maximum amount of the first asset to deposit. |
| `amount2` | `Option<crate::types::Amount>` | Maximum amount of the second asset to deposit. |
| `e_price` | `Option<crate::types::Amount>` | Effective price limit per LP token when using the `LimitLPToken` mode. |
| `lp_token_out` | `Option<crate::types::Amount>` | Exact number of LP tokens the depositor wants to receive. |
| `trading_fee` | `Option<u16>` | Trading fee vote to submit alongside the deposit (0-1000). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMVote`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Votes on the trading fee for an AMM pool.

LP token holders can vote to change the pool's trading fee. The effective fee is
the LP-token-weighted average of all active votes.

```rust
use xrpl::types::{Asset, transactions::amm::AMMVote};
let tx = AMMVote {
    asset: Asset::xrp(),
    asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    trading_fee: 500, // vote for 0.5%
};
```

```rust
pub struct AMMVote {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
    pub trading_fee: u16,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset of the pool. |
| `asset2` | `crate::types::Asset` | Second asset of the pool. |
| `trading_fee` | `u16` | Proposed trading fee in units of 1/100,000 of a percent (0-1000). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `AMMWithdraw`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Redeems LP tokens and withdraws assets from an AMM pool.

Supports several withdrawal modes selected by combining the optional fields and
transaction flags (e.g. single-asset, double-asset, or full withdrawal).

```rust
use xrpl::types::{Amount, Asset, transactions::amm::AMMWithdraw};
let tx = AMMWithdraw {
    asset: Asset::xrp(),
    asset2: Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh").unwrap(),
    lp_token_in: Some(Amount::IssuedCurrency {
        value: "100".to_string(),
        currency: "03930D02208264E2E40EC1B0C09E4DB96EE197B1".to_string(),
        issuer: "rAMMPool".to_string(),
    }),
    amount: None,
    amount2: None,
    e_price: None,
};
```

```rust
pub struct AMMWithdraw {
    pub asset: crate::types::Asset,
    pub asset2: crate::types::Asset,
    pub amount: Option<crate::types::Amount>,
    pub amount2: Option<crate::types::Amount>,
    pub e_price: Option<crate::types::Amount>,
    pub lp_token_in: Option<crate::types::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset` | `crate::types::Asset` | First asset of the pool. |
| `asset2` | `crate::types::Asset` | Second asset of the pool. |
| `amount` | `Option<crate::types::Amount>` | Minimum amount of the first asset to receive. |
| `amount2` | `Option<crate::types::Amount>` | Minimum amount of the second asset to receive. |
| `e_price` | `Option<crate::types::Amount>` | Effective price limit per LP token when using the `LimitLPToken` mode. |
| `lp_token_in` | `Option<crate::types::Amount>` | Exact number of LP tokens to redeem. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `clawback`

Clawback transaction type.

```rust
pub mod clawback { /* ... */ }
```

### Types

#### Struct `Clawback`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Reclaims issued currency or MPT tokens from a holder's balance.

Only available to issuers whose token was created with clawback enabled.
For trust-line tokens, set the `issuer` sub-field of `amount` to the holder's address.
For MPTs, use the `holder` field instead.

```rust
use xrpl::types::{Amount, transactions::clawback::Clawback};
let tx = Clawback {
    amount: Amount::IssuedCurrency {
        value: "100".to_string(),
        currency: "USD".to_string(),
        issuer: "rHolderAccount".to_string(),
    },
    holder: None,
};
```

```rust
pub struct Clawback {
    pub amount: crate::types::Amount,
    pub holder: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount to claw back; for trust-line tokens the `issuer` sub-field identifies the holder. |
| `holder` | `Option<String>` | Holder account when clawing back MPT balances. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `credential`

Credential transaction types (CredentialCreate, CredentialAccept, CredentialDelete).

```rust
pub mod credential { /* ... */ }
```

### Types

#### Struct `CredentialAccept`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Subject accepts a verifiable credential that was previously issued to them.

A credential only becomes active after it is accepted; unaccepted credentials
remain in a pending state on the ledger.

```rust
use xrpl::types::transactions::credential::CredentialAccept;
let tx = CredentialAccept {
    credential_type: Some(hex::encode("license")),
    issuer: Some("rIssuerAccount".to_string()),
};
```

```rust
pub struct CredentialAccept {
    pub credential_type: Option<String>,
    pub issuer: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `credential_type` | `Option<String>` | Hex-encoded credential type identifier. |
| `issuer` | `Option<String>` | Account that issued the credential. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `CredentialCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Issuer creates a verifiable credential for a subject account.

The credential must be accepted by the subject via `CredentialAccept` before it
is considered active. An optional expiration (Ripple epoch) and URI can be attached.

```rust
use xrpl::types::transactions::credential::CredentialCreate;
let tx = CredentialCreate {
    credential_type: Some(hex::encode("license")),
    subject: Some("rSubjectAccount".to_string()),
    expiration: None,
    uri: None,
};
```

```rust
pub struct CredentialCreate {
    pub credential_type: Option<String>,
    pub subject: Option<String>,
    pub expiration: Option<u32>,
    pub uri: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `credential_type` | `Option<String>` | Hex-encoded credential type identifier. |
| `subject` | `Option<String>` | Account the credential is issued to. |
| `expiration` | `Option<u32>` | Expiration time in seconds since the Ripple epoch (2000-01-01). |
| `uri` | `Option<String>` | Hex-encoded URI pointing to additional credential metadata. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `CredentialDelete`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Revokes or deletes a credential from the ledger.

Can be submitted by either the issuer or the subject. At least one of `subject` or
`issuer` must be provided to identify the credential entry.

```rust
use xrpl::types::transactions::credential::CredentialDelete;
let tx = CredentialDelete {
    credential_type: Some(hex::encode("license")),
    subject: Some("rSubjectAccount".to_string()),
    issuer: None,
};
```

```rust
pub struct CredentialDelete {
    pub credential_type: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `credential_type` | `Option<String>` | Hex-encoded credential type identifier. |
| `subject` | `Option<String>` | Subject account of the credential; required if `issuer` is not provided. |
| `issuer` | `Option<String>` | Issuer account of the credential; required if `subject` is not provided. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `did`

DID transaction types (DIDSet, DIDDelete).

```rust
pub mod did { /* ... */ }
```

### Types

#### Struct `DIDDelete`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Removes the Decentralized Identifier (DID) document associated with the submitting account.

```rust
use xrpl::types::transactions::did::DIDDelete;
let tx = DIDDelete {};
```

```rust
pub struct DIDDelete {
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `DIDSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates or updates the Decentralized Identifier (DID) document for the submitting account.

All three fields are optional - at least one must be provided. The `did_document`
and `data` fields must be hex-encoded.

```rust
use xrpl::types::transactions::did::DIDSet;
let tx = DIDSet {
    uri: Some("68747470733a2f2f6578616d706c652e636f6d2f646964".to_string()),
    did_document: None,
    data: None,
};
```

```rust
pub struct DIDSet {
    pub did_document: Option<String>,
    pub data: Option<String>,
    pub uri: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `did_document` | `Option<String>` | Hex-encoded W3C DID document. |
| `data` | `Option<String>` | Hex-encoded arbitrary data associated with the DID. |
| `uri` | `Option<String>` | Hex-encoded URI pointing to the DID document or related resource. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `escrow`

Escrow transaction types (EscrowCreate, EscrowFinish, EscrowCancel).

```rust
pub mod escrow { /* ... */ }
```

### Types

#### Struct `EscrowCancel`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Reclaims XRP from an expired escrow back to the owner.

Can be submitted by any account once the escrow's `CancelAfter` time has passed.

```rust
use xrpl::types::transactions::escrow::EscrowCancel;
let tx = EscrowCancel {
    owner: "rOwnerAccount".to_string(),
    offer_sequence: 42,
};
```

```rust
pub struct EscrowCancel {
    pub owner: String,
    pub offer_sequence: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `owner` | `String` | Account that created the escrow. |
| `offer_sequence` | `u32` | Sequence number of the `EscrowCreate` transaction that created the escrow. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `EscrowCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Locks XRP in escrow with an optional time-lock or crypto-condition for release.

The escrowed XRP is released when `EscrowFinish` is submitted (with the correct
fulfillment if a condition was set) and after `FinishAfter` has passed.

```rust
use xrpl::types::{Amount, transactions::escrow::EscrowCreate};
let tx = EscrowCreate {
    amount: Amount::Xrpl("10000000".to_string()),
    destination: "rRecipient".to_string(),
    finish_after: Some(946_684_800 + 86_400), // one day after Ripple epoch
    cancel_after: None,
    condition: None,
    destination_tag: None,
};
```

```rust
pub struct EscrowCreate {
    pub amount: crate::types::Amount,
    pub destination: String,
    pub cancel_after: Option<u32>,
    pub finish_after: Option<u32>,
    pub condition: Option<String>,
    pub destination_tag: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount of XRP (in drops) to lock in escrow. |
| `destination` | `String` | Account that receives the XRP when the escrow is finished. |
| `cancel_after` | `Option<u32>` | Ripple-epoch time after which the escrow can be cancelled. |
| `finish_after` | `Option<u32>` | Ripple-epoch time after which the escrow can be finished. |
| `condition` | `Option<String>` | PREIMAGE-SHA-256 crypto-condition (hex-encoded) that must be fulfilled to release funds. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `EscrowFinish`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Releases escrowed XRP to the destination account.

If the escrow has a crypto-condition, both `condition` and `fulfillment` must be
provided. The transaction can only succeed after `FinishAfter` has passed.

```rust
use xrpl::types::transactions::escrow::EscrowFinish;
let tx = EscrowFinish {
    owner: "rOwnerAccount".to_string(),
    offer_sequence: 42,
    condition: None,
    fulfillment: None,
};
```

```rust
pub struct EscrowFinish {
    pub owner: String,
    pub offer_sequence: u32,
    pub condition: Option<String>,
    pub fulfillment: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `owner` | `String` | Account that created the escrow. |
| `offer_sequence` | `u32` | Sequence number of the `EscrowCreate` transaction that created the escrow. |
| `condition` | `Option<String>` | The PREIMAGE-SHA-256 crypto-condition (hex-encoded) originally set on the escrow. |
| `fulfillment` | `Option<String>` | The fulfillment (hex-encoded) that satisfies the condition. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `mpt`

Multi-Purpose Token transaction types (MPTokenIssuanceCreate, MPTokenAuthorize, etc.).

```rust
pub mod mpt { /* ... */ }
```

### Types

#### Struct `MPTokenIssuanceCreateFlags`

Transaction flags for [`MPTokenIssuanceCreate`].

Combine flags with `|` and pass the result to `with_flags` on the builder:

```rust
use xrpl::types::MPTokenIssuanceCreateFlags as Flags;

let flags = Flags::CAN_TRANSFER | Flags::CAN_LOCK | Flags::CAN_CLAWBACK;
```

```rust
pub struct MPTokenIssuanceCreateFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `MPTokenAuthorizeFlags`

Transaction flags for [`MPTokenAuthorize`].

```rust
use xrpl::types::MPTokenAuthorizeFlags;

let flags = MPTokenAuthorizeFlags::UNAUTHORIZE;
```

```rust
pub struct MPTokenAuthorizeFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Enum `MPTokenIssuanceSetAction`

Action for [`MPTokenIssuanceSet`] - lock or unlock an issuance or holder balance.

```rust
use xrpl::types::MPTokenIssuanceSetAction;

let action = MPTokenIssuanceSetAction::Lock;
```

```rust
pub enum MPTokenIssuanceSetAction {
    Lock,
    Unlock,
}
```

##### Variants

###### `Lock`

Freeze the issuance or a specific holder's balance (`tfMPTLock`).

###### `Unlock`

Unfreeze the issuance or a specific holder's balance (`tfMPTUnlock`).

##### Implementations

###### Trait Implementations

#### Struct `MPTokenAuthorize`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Opts a holder in (or out) of an MPToken issuance.

A holder must authorize themselves before they can receive an MPT issuance.
When `tfMPTRequireAuth` is set on the issuance, the issuer uses this transaction
with `holder` populated to authorize individual accounts.

```rust
use xrpl::types::transactions::mpt::MPTokenAuthorize;
let tx = MPTokenAuthorize {
    mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
    holder: None, // omit when the holder self-authorizes
};
```

```rust
pub struct MPTokenAuthorize {
    pub mpt_issuance_id: String,
    pub holder: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mpt_issuance_id` | `String` | Identifier of the MPToken issuance. |
| `holder` | `Option<String>` | Account to authorize; omit when the transaction submitter is self-authorizing. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `MPTokenIssuanceCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates a new Multi-Purpose Token (MPT) issuance class on the ledger.

The submitting account becomes the issuer. Flags passed via the transaction's
`Flags` field control transferability, clawback, and authorization requirements.

```rust
use xrpl::types::transactions::mpt::MPTokenIssuanceCreate;
let tx = MPTokenIssuanceCreate {
    asset_scale: Some(2),
    maximum_amount: Some("1000000".to_string()),
    mpt_metadata: None,
    transfer_fee: Some(500), // 0.5%
};
```

```rust
pub struct MPTokenIssuanceCreate {
    pub asset_scale: Option<u8>,
    pub maximum_amount: Option<String>,
    pub mpt_metadata: Option<String>,
    pub transfer_fee: Option<u16>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset_scale` | `Option<u8>` | Number of decimal places (e.g. 2 means amounts are in hundredths). |
| `maximum_amount` | `Option<String>` | Maximum number of tokens that may be distributed (UInt64 as a decimal string). |
| `mpt_metadata` | `Option<String>` | Hex-encoded metadata associated with the issuance. |
| `transfer_fee` | `Option<u16>` | Transfer fee in units of 1/100,000 of a percent (0-50000). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `MPTokenIssuanceDestroy`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Destroys an MPToken issuance that has an outstanding balance of zero.

Once destroyed, the issuance ID is permanently removed from the ledger.

```rust
use xrpl::types::transactions::mpt::MPTokenIssuanceDestroy;
let tx = MPTokenIssuanceDestroy {
    mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
};
```

```rust
pub struct MPTokenIssuanceDestroy {
    pub mpt_issuance_id: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mpt_issuance_id` | `String` | Identifier of the MPToken issuance to destroy. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `MPTokenIssuanceSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Locks or unlocks an MPToken issuance or a specific holder's balance.

Use flag `tfMPTLock` to lock and `tfMPTUnlock` to unlock. To target a single
holder's balance, provide the `holder` field; otherwise the entire issuance is affected.

```rust
use xrpl::types::transactions::mpt::MPTokenIssuanceSet;
let tx = MPTokenIssuanceSet {
    mpt_issuance_id: "0000000024B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48".to_string(),
    holder: None, // omit to lock/unlock the entire issuance
};
```

```rust
pub struct MPTokenIssuanceSet {
    pub mpt_issuance_id: String,
    pub holder: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mpt_issuance_id` | `String` | Identifier of the MPToken issuance to configure. |
| `holder` | `Option<String>` | Specific holder account to lock or unlock; omit to affect the whole issuance. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `nft`

NFToken transaction types (NFTokenMint, NFTokenBurn, NFTokenCreateOffer, etc.).

```rust
pub mod nft { /* ... */ }
```

### Types

#### Struct `NFTokenMintFlags`

Transaction flags for [`NFTokenMint`].

```rust
use xrpl::types::NFTokenMintFlags as Flags;

let flags = Flags::BURNABLE | Flags::TRANSFERABLE;
```

```rust
pub struct NFTokenMintFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `NFTokenCreateOfferFlags`

Transaction flags for [`NFTokenCreateOffer`].

```rust
use xrpl::types::NFTokenCreateOfferFlags;

let flags = NFTokenCreateOfferFlags::SELL;
```

```rust
pub struct NFTokenCreateOfferFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

#### Struct `NFTokenAcceptOffer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Completes an NFT trade by accepting an existing buy or sell offer.

In brokered mode, supply both a sell and a buy offer; the difference minus
`NFTokenBrokerFee` goes to the broker.

```rust
use xrpl::types::transactions::nft::NFTokenAcceptOffer;
let tx = NFTokenAcceptOffer {
    nftoken_sell_offer: Some("offer_id_hex".to_string()),
    nftoken_buy_offer: None,
    nftoken_broker_fee: None,
};
```

```rust
pub struct NFTokenAcceptOffer {
    pub nftoken_sell_offer: Option<String>,
    pub nftoken_buy_offer: Option<String>,
    pub nftoken_broker_fee: Option<crate::types::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nftoken_sell_offer` | `Option<String>` | Ledger object ID of the NFT sell offer to accept. |
| `nftoken_buy_offer` | `Option<String>` | Ledger object ID of the NFT buy offer to accept. |
| `nftoken_broker_fee` | `Option<crate::types::Amount>` | Fee retained by the broker in brokered-mode transactions. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `NFTokenBurn`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Permanently destroys an NFToken, removing it from the ledger.

The submitter must be the token owner, or the issuer if the token was minted with
the `tfBurnable` flag. Once burned, the token ID cannot be reused.

```rust
use xrpl::types::transactions::nft::NFTokenBurn;
let tx = NFTokenBurn {
    nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
    owner: None,
};
```

```rust
pub struct NFTokenBurn {
    pub nftoken_id: String,
    pub owner: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nftoken_id` | `String` | The 256-bit identifier of the NFToken to burn. |
| `owner` | `Option<String>` | Current owner, required when the issuer (not the owner) is submitting the burn. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `NFTokenCancelOffer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Cancels one or more open NFT buy or sell offers.

Multiple offer IDs can be cancelled in a single transaction. The submitter must be
either the offer creator or the NFT issuer (for offers on non-transferable tokens).

```rust
use xrpl::types::transactions::nft::NFTokenCancelOffer;
let tx = NFTokenCancelOffer {
    nftoken_offers: vec!["offer_id_1".to_string(), "offer_id_2".to_string()],
};
```

```rust
pub struct NFTokenCancelOffer {
    pub nftoken_offers: Vec<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nftoken_offers` | `Vec<String>` | List of ledger object IDs of NFT offers to cancel. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `NFTokenCreateOffer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates a buy or sell offer for an NFToken.

Set the `tfSellNFToken` flag to create a sell offer; omit it for a buy offer.
For buy offers, `owner` must identify the current token holder.

```rust
use xrpl::types::{Amount, transactions::nft::NFTokenCreateOffer};
let tx = NFTokenCreateOffer {
    nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
    amount: Amount::Xrpl("10000000".to_string()),
    owner: None,
    expiration: None,
    destination: None,
};
```

```rust
pub struct NFTokenCreateOffer {
    pub nftoken_id: String,
    pub amount: crate::types::Amount,
    pub owner: Option<String>,
    pub expiration: Option<u32>,
    pub destination: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nftoken_id` | `String` | The 256-bit identifier of the NFToken. |
| `amount` | `crate::types::Amount` | Offered price (XRP or issued currency). |
| `owner` | `Option<String>` | Current token owner; required for buy offers where the submitter is not the owner. |
| `expiration` | `Option<u32>` | Ripple-epoch time after which the offer expires. |
| `destination` | `Option<String>` | Restricts acceptance to a specific account; omit to allow any account. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `NFTokenMint`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Mints a new NFToken and places it in the submitter's NFToken page.

Set the `tfTransferable` flag to allow the token to be sold or transferred.
The royalty (`transfer_fee`) is enforced by the protocol on every secondary sale.

```rust
use xrpl::types::transactions::nft::NFTokenMint;
let tx = NFTokenMint {
    nftoken_taxon: 0,
    issuer: None,
    transfer_fee: Some(5000), // 5% royalty
    uri: Some("68747470733a2f2f6578616d706c652e636f6d2f6e6674".to_string()),
};
```

```rust
pub struct NFTokenMint {
    pub nftoken_taxon: u32,
    pub issuer: Option<String>,
    pub transfer_fee: Option<u16>,
    pub uri: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nftoken_taxon` | `u32` | Arbitrary taxon (collection identifier) chosen by the issuer. |
| `issuer` | `Option<String>` | Issuer account, if minting on behalf of another account (requires `NFTokenMinter` to be set). |
| `transfer_fee` | `Option<u16>` | Royalty fee in units of 1/100,000 of a percent (0-50000). |
| `uri` | `Option<String>` | Hex-encoded URI pointing to the token's metadata (max 512 characters). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `offer`

DEX offer transaction types (OfferCreate, OfferCancel).

```rust
pub mod offer { /* ... */ }
```

### Types

#### Struct `OfferCreateFlags`

Transaction flags for [`OfferCreate`].

```rust
use xrpl::types::OfferCreateFlags as Flags;

let flags = Flags::PASSIVE | Flags::SELL;
```

```rust
pub struct OfferCreateFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `OfferCancel`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Cancels an open limit order on the XRPL decentralized exchange by sequence number.

```rust
use xrpl::types::transactions::offer::OfferCancel;
let tx = OfferCancel { offer_sequence: 42 };
```

```rust
pub struct OfferCancel {
    pub offer_sequence: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `offer_sequence` | `u32` | Sequence number of the `OfferCreate` transaction that placed the order. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `OfferCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Places a limit order on the XRPL decentralized exchange.

The order is filled immediately against existing offers in the order book at
as-good-or-better rates. Any unfilled remainder is placed on the book unless
the `tfImmediateOrCancel` or `tfFillOrKill` flags are set.

```rust
use xrpl::types::{Amount, transactions::offer::OfferCreate};
let tx = OfferCreate {
    taker_gets: Amount::Xrpl("1000000".to_string()),
    taker_pays: Amount::IssuedCurrency {
        value: "1".to_string(),
        currency: "USD".to_string(),
        issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
    },
    expiration: None,
    offer_sequence: None,
};
```

```rust
pub struct OfferCreate {
    pub expiration: Option<u32>,
    pub offer_sequence: Option<u32>,
    pub taker_gets: crate::types::Amount,
    pub taker_pays: crate::types::Amount,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `expiration` | `Option<u32>` | Ripple-epoch time after which the offer is automatically invalidated. |
| `offer_sequence` | `Option<u32>` | Sequence number of an existing offer to cancel when this offer is placed. |
| `taker_gets` | `crate::types::Amount` | Amount the taker receives (what the submitter gives up). |
| `taker_pays` | `crate::types::Amount` | Amount the taker pays (what the submitter receives). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `oracle`

Price oracle transaction types (OracleSet, OracleDelete).

```rust
pub mod oracle { /* ... */ }
```

### Types

#### Struct `OracleDelete`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Removes an on-ledger price oracle document.

Only the oracle's owner account can delete it. Once deleted, the document ID
can be reused.

```rust
use xrpl::types::transactions::oracle::OracleDelete;
let tx = OracleDelete { oracle_document_id: 1 };
```

```rust
pub struct OracleDelete {
    pub oracle_document_id: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `oracle_document_id` | `u32` | Unique identifier of the oracle document to remove. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `OracleSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates or updates an on-ledger price oracle with a series of price data entries.

Each entry in `price_data_series` represents a base/quote asset pair. A new
`oracle_document_id` creates the oracle; an existing ID updates it.

```rust
use xrpl::types::transactions::oracle::{OracleSet, PriceDataWrapper, PriceData};
let tx = OracleSet {
    oracle_document_id: 1,
    last_update_time: 946_684_800,
    price_data_series: vec![],
    asset_class: Some("currency".to_string()),
    provider: None,
    uri: None,
};
```

```rust
pub struct OracleSet {
    pub asset_class: Option<String>,
    pub last_update_time: u32,
    pub oracle_document_id: u32,
    pub price_data_series: Vec<PriceDataWrapper>,
    pub provider: Option<String>,
    pub uri: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset_class` | `Option<String>` | Hex-encoded string describing the asset class (e.g. `"currency"`). |
| `last_update_time` | `u32` | Timestamp of the most recent price update (seconds since the Ripple epoch). |
| `oracle_document_id` | `u32` | Unique identifier for this oracle document (created or updated). |
| `price_data_series` | `Vec<PriceDataWrapper>` | One or more base/quote price entries. |
| `provider` | `Option<String>` | Hex-encoded name or identifier of the price data provider. |
| `uri` | `Option<String>` | Hex-encoded URI pointing to additional oracle metadata. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `PriceDataWrapper`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Wire-format wrapper that nests `PriceData` under the `PriceData` key.

Required by the XRPL JSON protocol; use [`PriceData`] for the actual price entry.

```rust
pub struct PriceDataWrapper {
    pub price_data: PriceData,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `price_data` | `PriceData` | The price data entry. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `PriceData`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

One base/quote price entry within an oracle's `PriceDataSeries`.

The raw integer price is stored in `asset_price` (as a decimal string) and
scaled by `10^-scale` to get the actual value.

```rust
use xrpl::types::transactions::oracle::PriceData;
let entry = PriceData {
    base_asset: "XRP".to_string(),
    quote_asset: "USD".to_string(),
    asset_price: Some("5000".to_string()), // 0.50 USD with scale=4
    scale: Some(4),
};
```

```rust
pub struct PriceData {
    pub asset_price: Option<String>,
    pub base_asset: String,
    pub quote_asset: String,
    pub scale: Option<u8>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `asset_price` | `Option<String>` | Raw price as a UInt64 decimal string (actual value = asset_price x 10^-scale). |
| `base_asset` | `String` | Currency code or asset symbol for the base asset. |
| `quote_asset` | `String` | Currency code or asset symbol for the quote asset. |
| `scale` | `Option<u8>` | Number of decimal places to apply to `asset_price` (0-10). |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(base_asset: impl AsRef<str>, quote_asset: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a `PriceData` entry with only the required base/quote assets.

- ```rust
  pub fn with_price</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, asset_price: impl AsRef<str>, scale: u8) -> Self { /* ... */ }
  ```
  Attaches a raw price and the decimal scale to apply to it.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `payment`

Payment and check transaction types (Payment, CheckCreate, CheckCash, CheckCancel).

```rust
pub mod payment { /* ... */ }
```

### Types

#### Enum `PaymentFlag`

**Attributes:**

- `NonExhaustive`

An individual flag for [`Payment`] transactions.

Use with [`PaymentFlags`] to build or inspect the `Flags` bitmask.

# Examples

```rust
use xrpl::types::{PaymentFlag, PaymentFlags};

// Building flags for a partial payment:
let flags = PaymentFlags::from(PaymentFlag::PartialPayment);

// Inspecting flags on a received transaction:
let raw: u32 = 0x00020000;
assert!(PaymentFlags::from(raw).has(PaymentFlag::PartialPayment));
```

```rust
pub enum PaymentFlag {
    NoRippleDirect,
    PartialPayment,
    LimitQuality,
    Unknown(u32),
}
```

##### Variants

###### `NoRippleDirect`

Only use paths in the `paths` field; skip the default ripple path.

###### `PartialPayment`

Allow delivery of less than the full `amount`; always check `delivered_amount` in metadata.

###### `LimitQuality`

Only take paths where all offers meet or exceed the `send_max` quality ratio.

###### `Unknown`

An unrecognized flag from a protocol amendment not yet reflected in this library.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn mask(self: Self) -> u32 { /* ... */ }
  ```
  The bitmask value for this flag as used in the `Flags` field.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> PaymentFlags { /* ... */ }
    ```

  - ```rust
    fn bitor(self: Self, rhs: PaymentFlag) -> Self { /* ... */ }
    ```

#### Struct `PaymentFlags`

The `Flags` bitmask for a [`Payment`] transaction.

Use [`has`](Self::has) to check individual flags on incoming transactions,
and [`From<PaymentFlag>`] or [`BitOr`] to build a flags value for the builder.

# Examples

```rust
use xrpl::types::{PaymentFlag, PaymentFlags};

// Combining flags for the builder:
let flags = PaymentFlag::PartialPayment | PaymentFlag::LimitQuality;

// Reading flags from a received transaction:
let flags = PaymentFlags::from(0x00020000_u32);
assert!(flags.has(PaymentFlag::PartialPayment));
assert!(!flags.has(PaymentFlag::LimitQuality));
```

```rust
pub struct PaymentFlags(/* private field */);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `private` | *Private field* |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: PaymentFlag) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

- ```rust
  pub fn raw(self: Self) -> u32 { /* ... */ }
  ```
  The raw bitmask value.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

  - ```rust
    fn bitor(self: Self, rhs: PaymentFlag) -> Self { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<D: Deserializer<''de>>(d: D) -> Result<Self, <D as >::Error> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<S: Serializer>(self: &Self, s: S) -> Result<<S as >::Ok, <S as >::Error> { /* ... */ }
    ```

#### Struct `Payment`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Sends XRP or issued currency to a destination account.

Supports direct XRP payments, issued-currency payments through trust lines,
and cross-currency payments via `paths`. Always verify `meta.delivered_amount`
rather than `amount` to guard against partial payment attacks.

```rust
use xrpl::types::{Amount, transactions::payment::Payment};
let tx = Payment {
    amount: Some(Amount::Xrpl("1000000".to_string())),
    deliver_max: Some(Amount::Xrpl("1000000".to_string())),
    destination: "rRecipient".to_string(),
    deliver_min: None,
    destination_tag: None,
    invoice_id: None,
    paths: None,
    send_max: None,
};
```

```rust
pub struct Payment {
    pub amount: Option<crate::types::Amount>,
    pub deliver_max: Option<crate::types::Amount>,
    pub deliver_min: Option<crate::types::Amount>,
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub invoice_id: Option<String>,
    pub paths: Option<Vec<Vec<PathStep>>>,
    pub send_max: Option<crate::types::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `Option<crate::types::Amount>` | The amount to deliver to the destination (for direct or single-currency payments). |
| `deliver_max` | `Option<crate::types::Amount>` | Maximum amount to deliver; used in cross-currency or partial-payment scenarios. |
| `deliver_min` | `Option<crate::types::Amount>` | Minimum amount to deliver when `tfPartialPayment` is set. |
| `destination` | `String` | The account that receives the payment. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `invoice_id` | `Option<String>` | 64-character hex invoice identifier for reconciliation. |
| `paths` | `Option<Vec<Vec<PathStep>>>` | Paths for cross-currency payments; each path is an ordered list of `PathStep` hops. |
| `send_max` | `Option<crate::types::Amount>` | Maximum amount the sender is willing to spend (for cross-currency payments). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `CheckCancel`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Voids an uncashed check, removing it from the ledger.

Can be submitted by either the check sender or the intended recipient.

```rust
use xrpl::types::transactions::payment::CheckCancel;
let tx = CheckCancel {
    check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
};
```

```rust
pub struct CheckCancel {
    pub check_id: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `check_id` | `String` | Ledger object ID of the check to cancel. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `CheckCash`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Redeems a check to receive funds from the check sender's account.

Exactly one of `amount` (exact delivery) or `deliver_min` (flexible delivery) must
be provided. Only the check's intended recipient can cash it.

```rust
use xrpl::types::{Amount, transactions::payment::CheckCash};
let tx = CheckCash {
    check_id: "838766BA2B995C00744175F69A1B11E32C3DBC40E64801A4056FCBD657F57334".to_string(),
    amount: Some(Amount::Xrpl("1000000".to_string())),
    deliver_min: None,
};
```

```rust
pub struct CheckCash {
    pub check_id: String,
    pub amount: Option<crate::types::Amount>,
    pub deliver_min: Option<crate::types::Amount>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `check_id` | `String` | Ledger object ID of the check to cash. |
| `amount` | `Option<crate::types::Amount>` | Exact amount to receive; mutually exclusive with `deliver_min`. |
| `deliver_min` | `Option<crate::types::Amount>` | Minimum amount to receive, allowing the ledger to deliver up to the check's `SendMax`. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `CheckCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates a deferred payment authorization (a "check") that the recipient may cash later.

Similar to a paper check: the sender pre-authorizes up to `send_max`, and the
recipient cashes it at any time before expiry.

```rust
use xrpl::types::{Amount, transactions::payment::CheckCreate};
let tx = CheckCreate {
    destination: "rRecipient".to_string(),
    send_max: Amount::Xrpl("10000000".to_string()),
    destination_tag: None,
    expiration: None,
    invoice_id: None,
};
```

```rust
pub struct CheckCreate {
    pub destination: String,
    pub send_max: crate::types::Amount,
    pub destination_tag: Option<u32>,
    pub expiration: Option<u32>,
    pub invoice_id: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `destination` | `String` | Account authorized to cash the check. |
| `send_max` | `crate::types::Amount` | Maximum amount the sender is willing to pay when the check is cashed. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `expiration` | `Option<u32>` | Ripple-epoch time after which the check can no longer be cashed. |
| `invoice_id` | `Option<String>` | 64-character hex invoice identifier for reconciliation. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `PathStep`

One intermediate hop in a cross-currency payment path.

Each step specifies the account (rippling through), currency, or issuer at that
point in the path. The combination of fields determines what type of hop it is.

```rust
use xrpl::types::transactions::payment::PathStep;
let hop = PathStep {
    account: Some("rIntermediaryAccount".to_string()),
    currency: None,
    issuer: None,
};
```

```rust
pub struct PathStep {
    pub account: Option<String>,
    pub currency: Option<String>,
    pub issuer: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `Option<String>` | Intermediate account to ripple through. |
| `currency` | `Option<String>` | Currency to convert into at this step. |
| `issuer` | `Option<String>` | Issuer of the currency at this step. |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Creates an empty path step. Chain `with_*` methods to populate the

- ```rust
  pub fn with_account</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, account: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the intermediate account.

- ```rust
  pub fn with_currency</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, currency: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the currency code.

- ```rust
  pub fn with_issuer</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, issuer: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the currency issuer.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

## Module `payment_channel`

Payment channel transaction types (PaymentChannelCreate, PaymentChannelFund, PaymentChannelClaim).

```rust
pub mod payment_channel { /* ... */ }
```

### Types

#### Enum `PaymentChannelClaimAction`

Action for [`PaymentChannelClaim`] - close the channel or renew its settlement delay.

```rust
use xrpl::types::PaymentChannelClaimAction;

let action = PaymentChannelClaimAction::Close;
```

```rust
pub enum PaymentChannelClaimAction {
    Close,
    Renew,
}
```

##### Variants

###### `Close`

Request to close the channel after the settlement delay (`tfClose`).

###### `Renew`

Reset the channel's expiry to now + settle delay (`tfRenew`).

##### Implementations

###### Trait Implementations

#### Struct `PaymentChannelClaim`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Redeems a signed claim from a payment channel to receive XRP.

Either the sender or the recipient can submit this transaction. To close the
channel, set the `tfClose` flag. To renew the settlement delay, set `tfRenew`.

```rust
use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelClaim};
let tx = PaymentChannelClaim {
    channel: "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
    amount: Some(Amount::Xrpl("1000000".to_string())),
    balance: None,
    credential_ids: None,
    public_key: None,
    signature: None,
};
```

```rust
pub struct PaymentChannelClaim {
    pub channel: String,
    pub amount: Option<crate::types::Amount>,
    pub balance: Option<crate::types::Amount>,
    pub credential_ids: Option<Vec<String>>,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `channel` | `String` | 256-bit ledger object ID of the payment channel. |
| `amount` | `Option<crate::types::Amount>` | Total XRP (drops) that the channel can pay out after this claim. |
| `balance` | `Option<crate::types::Amount>` | Total XRP (drops) delivered by the channel so far (cumulative). |
| `credential_ids` | `Option<Vec<String>>` | Credential IDs used to satisfy deposit authorization on the destination. |
| `public_key` | `Option<String>` | Sender's secp256k1 or Ed25519 public key used to verify the signature. |
| `signature` | `Option<String>` | Sender's signature authorizing the claim amount. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `PaymentChannelCreate`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Opens a unidirectional XRP payment channel between two accounts.

The `settle_delay` enforces a waiting period after the channel is closed before
the sender can reclaim unclaimed XRP. The `public_key` must match the key used
to sign off-ledger claim messages.

```rust
use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelCreate};
let tx = PaymentChannelCreate {
    amount: Amount::Xrpl("100000000".to_string()),
    destination: "rRecipient".to_string(),
    public_key: "ED...".to_string(),
    settle_delay: 3600,
    destination_tag: None,
    cancel_after: None,
};
```

```rust
pub struct PaymentChannelCreate {
    pub amount: crate::types::Amount,
    pub destination: String,
    pub public_key: String,
    pub settle_delay: u32,
    pub destination_tag: Option<u32>,
    pub cancel_after: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount of XRP (drops) to fund the channel with. |
| `destination` | `String` | Account that can receive XRP from this channel. |
| `public_key` | `String` | Sender's public key for verifying off-ledger claim signatures. |
| `settle_delay` | `u32` | Seconds the channel must remain open after a close request before XRP can be reclaimed. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `cancel_after` | `Option<u32>` | Ripple-epoch time after which the channel can be closed by anyone. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `PaymentChannelFund`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Adds more XRP to an open payment channel or extends its expiry.

Only the channel's source account can fund it. Optionally set or extend the
channel's expiration (must be after the current expiration if already set).

```rust
use xrpl::types::{Amount, transactions::payment_channel::PaymentChannelFund};
let tx = PaymentChannelFund {
    channel: "5DB01B7FFED6B67E6B0414DED11E051D2EE2B7619CE0EAA6286D67A3A4D5BDB3".to_string(),
    amount: Amount::Xrpl("10000000".to_string()),
    expiration: None,
};
```

```rust
pub struct PaymentChannelFund {
    pub channel: String,
    pub amount: crate::types::Amount,
    pub expiration: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `channel` | `String` | 256-bit ledger object ID of the channel to fund. |
| `amount` | `crate::types::Amount` | Additional XRP (drops) to deposit into the channel. |
| `expiration` | `Option<u32>` | New Ripple-epoch expiration time for the channel; must be later than the current expiry. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `trust_set`

TrustSet transaction type.

```rust
pub mod trust_set { /* ... */ }
```

### Types

#### Struct `TrustSetFlags`

Transaction flags for [`TrustSet`].

```rust
use xrpl::types::TrustSetFlags as Flags;

let flags = Flags::SET_NO_RIPPLE | Flags::SET_FREEZE;
```

```rust
pub struct TrustSetFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `TrustSet`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Creates or modifies a trust line between the submitter and a currency issuer.

Setting `limit_amount` to zero with no outstanding balance closes the trust line.
Use the `tfSetNoRipple` / `tfClearNoRipple` flags to control rippling behavior.

```rust
use xrpl::types::{Amount, transactions::trust_set::TrustSet};
let tx = TrustSet {
    limit_amount: Amount::IssuedCurrency {
        value: "1000".to_string(),
        currency: "USD".to_string(),
        issuer: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
    },
    quality_in: None,
    quality_out: None,
};
```

```rust
pub struct TrustSet {
    pub limit_amount: crate::types::Amount,
    pub quality_in: Option<u32>,
    pub quality_out: Option<u32>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `limit_amount` | `crate::types::Amount` | Maximum amount of the issued currency the submitter is willing to hold; defines the trust line. |
| `quality_in` | `Option<u32>` | Incoming exchange rate applied to balances flowing in through this trust line (billionths). |
| `quality_out` | `Option<u32>` | Outgoing exchange rate applied to balances flowing out through this trust line (billionths). |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

## Module `xchain`

Cross-chain bridge transaction types (XChainCreateBridge, XChainCommit, XChainClaim, etc.).

```rust
pub mod xchain { /* ... */ }
```

### Types

#### Struct `XChainModifyBridgeFlags`

Transaction flags for [`XChainModifyBridge`].

```rust
use xrpl::types::XChainModifyBridgeFlags as Flags;

let flags = Flags::CLEAR_ACCOUNT_CREATE_AMOUNT;
```

```rust
pub struct XChainModifyBridgeFlags(pub u32);
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `u32` |  |

##### Implementations

###### Methods

- ```rust
  pub fn has(self: Self, flag: Self) -> bool { /* ... */ }
  ```
  Returns `true` if the given flag is set in this bitmask.

###### Trait Implementations

- **BitOr**
  - ```rust
    fn bitor(self: Self, rhs: Self) -> Self { /* ... */ }
    ```

#### Struct `XChainAccountCreateCommit`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Funds the creation of an account on the destination chain via a cross-chain bridge.

Used when the target account does not yet exist on the other chain. Witness servers
observe this transaction and attest with `XChainAddAccountCreateAttestation`.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAccountCreateCommit};
let tx = XChainAccountCreateCommit {
    amount: Amount::Xrpl("20000000".to_string()),
    destination: "rNewAccount".to_string(),
    signature_reward: Amount::Xrpl("100000000".to_string()),
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
};
```

```rust
pub struct XChainAccountCreateCommit {
    pub amount: crate::types::Amount,
    pub destination: String,
    pub signature_reward: crate::types::Amount,
    pub xchain_bridge: crate::types::XChainBridge,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | XRP (or token) amount to send to fund the new account on the other chain. |
| `destination` | `String` | Account to create on the destination chain. |
| `signature_reward` | `crate::types::Amount` | Reward paid to witness servers for attesting this transaction. |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration identifying the two chains and door accounts. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainAddAccountCreateAttestation`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Witness server attestation that an `XChainAccountCreateCommit` occurred on the source chain.

Submitted by each witness server individually. Once a quorum of attestations is
collected, the destination-chain account creation is finalized.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAddAccountCreateAttestation};
// Typically constructed by a witness server; fields are sourced from the source-chain event.
```

```rust
pub struct XChainAddAccountCreateAttestation {
    pub amount: crate::types::Amount,
    pub attestation_reward_account: String,
    pub attestation_signer_account: String,
    pub destination: String,
    pub other_chain_source: String,
    pub public_key: String,
    pub signature: String,
    pub signature_reward: crate::types::Amount,
    pub was_locking_chain_send: u8,
    pub xchain_account_create_count: String,
    pub xchain_bridge: crate::types::XChainBridge,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount committed on the source chain. |
| `attestation_reward_account` | `String` | Account that receives the witness reward on the destination chain. |
| `attestation_signer_account` | `String` | Account whose key signed the attestation. |
| `destination` | `String` | Destination account to be created on the issuing chain. |
| `other_chain_source` | `String` | Source account on the locking chain that submitted the commit. |
| `public_key` | `String` | Public key of the witness signer. |
| `signature` | `String` | Witness server's signature over the attestation data. |
| `signature_reward` | `crate::types::Amount` | Reward paid to the witness for this attestation. |
| `was_locking_chain_send` | `u8` | `1` if the commit originated from the locking chain, `0` from the issuing chain. |
| `xchain_account_create_count` | `String` | Sequential counter for cross-chain account creation events (UInt64 as decimal string). |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainAddClaimAttestation`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Witness server attestation that an `XChainCommit` occurred on the source chain.

Submitted by each witness server individually. Once a quorum of attestations is
collected for a given `xchain_claim_id`, the destination-chain `XChainClaim`
can succeed.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainAddClaimAttestation};
// Typically constructed by a witness server; fields are sourced from the source-chain event.
```

```rust
pub struct XChainAddClaimAttestation {
    pub amount: crate::types::Amount,
    pub attestation_reward_account: String,
    pub attestation_signer_account: String,
    pub destination: Option<String>,
    pub other_chain_source: String,
    pub public_key: String,
    pub signature: String,
    pub was_locking_chain_send: u8,
    pub xchain_bridge: crate::types::XChainBridge,
    pub xchain_claim_id: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount committed on the source chain. |
| `attestation_reward_account` | `String` | Account that receives the witness reward on the destination chain. |
| `attestation_signer_account` | `String` | Account whose key signed the attestation. |
| `destination` | `Option<String>` | Optional destination account override on the destination chain. |
| `other_chain_source` | `String` | Source account on the origin chain that submitted the `XChainCommit`. |
| `public_key` | `String` | Public key of the witness signer. |
| `signature` | `String` | Witness server's signature over the attestation data. |
| `was_locking_chain_send` | `u8` | `1` if the commit originated from the locking chain, `0` from the issuing chain. |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration. |
| `xchain_claim_id` | `String` | Claim ID that corresponds to the `XChainCreateClaimID` on the destination chain. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainClaim`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Completes a cross-chain transfer by claiming the committed assets on the destination chain.

Succeeds only after enough witness attestations have been submitted for the
associated `xchain_claim_id`.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainClaim};
let tx = XChainClaim {
    amount: Amount::Xrpl("100000000".to_string()),
    destination: "rDestination".to_string(),
    destination_tag: None,
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
    xchain_claim_id: "1".to_string(),
};
```

```rust
pub struct XChainClaim {
    pub amount: crate::types::Amount,
    pub destination: String,
    pub destination_tag: Option<u32>,
    pub xchain_bridge: crate::types::XChainBridge,
    pub xchain_claim_id: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount to receive on the destination chain. |
| `destination` | `String` | Account on the destination chain that receives the assets. |
| `destination_tag` | `Option<u32>` | Destination tag for routing within the destination account. |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration. |
| `xchain_claim_id` | `String` | Claim ID created by `XChainCreateClaimID` on this chain. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainCommit`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Locks or burns assets on the source chain to initiate a cross-chain transfer.

The `xchain_claim_id` must be obtained in advance via `XChainCreateClaimID` on
the destination chain. Witness servers observe this and submit attestations.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCommit};
let tx = XChainCommit {
    amount: Amount::Xrpl("100000000".to_string()),
    other_chain_destination: Some("rDestination".to_string()),
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
    xchain_claim_id: "1".to_string(),
};
```

```rust
pub struct XChainCommit {
    pub amount: crate::types::Amount,
    pub other_chain_destination: Option<String>,
    pub xchain_bridge: crate::types::XChainBridge,
    pub xchain_claim_id: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amount` | `crate::types::Amount` | Amount of XRP or tokens to lock on the source chain. |
| `other_chain_destination` | `Option<String>` | Destination account on the other chain (overrides the one in `XChainClaim` if set). |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration. |
| `xchain_claim_id` | `String` | Claim ID obtained from `XChainCreateClaimID` on the destination chain. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainCreateBridge`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Registers a new cross-chain bridge on the ledger.

Must be submitted by the door account on both the locking chain and the issuing chain.
The `signature_reward` is distributed to witness servers for each attestation.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCreateBridge};
let tx = XChainCreateBridge {
    signature_reward: Amount::Xrpl("100000000".to_string()),
    min_account_create_amount: None,
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
};
```

```rust
pub struct XChainCreateBridge {
    pub signature_reward: crate::types::Amount,
    pub min_account_create_amount: Option<crate::types::Amount>,
    pub xchain_bridge: crate::types::XChainBridge,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signature_reward` | `crate::types::Amount` | Total reward paid to witness servers per attestation batch. |
| `min_account_create_amount` | `Option<crate::types::Amount>` | Minimum XRP required when creating an account on the issuing chain via this bridge. |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration identifying the two chains and door accounts. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainCreateClaimID`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Reserves a cross-chain claim ID slot on the destination chain before a transfer begins.

The resulting claim ID must be included in the corresponding `XChainCommit` on the
source chain. One claim ID is consumed per cross-chain transfer.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainCreateClaimID};
let tx = XChainCreateClaimID {
    other_chain_source: "rSourceAccount".to_string(),
    signature_reward: Amount::Xrpl("100000000".to_string()),
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
};
```

```rust
pub struct XChainCreateClaimID {
    pub other_chain_source: String,
    pub signature_reward: crate::types::Amount,
    pub xchain_bridge: crate::types::XChainBridge,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `other_chain_source` | `String` | Account on the source chain that will submit the `XChainCommit`. |
| `signature_reward` | `crate::types::Amount` | Reward amount paid to witness servers (must match the bridge's `SignatureReward`). |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

#### Struct `XChainModifyBridge`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Updates parameters of an existing cross-chain bridge.

Only the door account that originally created the bridge can modify it.
At least one of `signature_reward` or `min_account_create_amount` must be provided.

```rust
use xrpl::types::{Amount, Asset, XChainBridge, transactions::xchain::XChainModifyBridge};
let tx = XChainModifyBridge {
    signature_reward: Some(Amount::Xrpl("200000000".to_string())),
    min_account_create_amount: None,
    xchain_bridge: XChainBridge {
        locking_chain_door: "rLockDoor".to_string(),
        locking_chain_issue: Asset::xrp(),
        issuing_chain_door: "rIssueDoor".to_string(),
        issuing_chain_issue: Asset::xrp(),
    },
};
```

```rust
pub struct XChainModifyBridge {
    pub signature_reward: Option<crate::types::Amount>,
    pub min_account_create_amount: Option<crate::types::Amount>,
    pub xchain_bridge: crate::types::XChainBridge,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signature_reward` | `Option<crate::types::Amount>` | New total reward paid to witness servers per attestation batch. |
| `min_account_create_amount` | `Option<crate::types::Amount>` | New minimum XRP required to create an account on the issuing chain via this bridge. |
| `xchain_bridge` | `crate::types::XChainBridge` | Bridge configuration identifying this bridge. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **TransactionTypeBuilder**
  - ```rust
    fn validate(self: &Self) -> Result<(), BuildError> { /* ... */ }
    ```

  - ```rust
    fn build_transaction_type(self: Self) -> Result<<Self as >::TransactionType, BuildError> { /* ... */ }
    ```

### Types

#### Enum `TransactionType`

**Attributes:**

- `NonExhaustive`

Discriminated union over every XRPL transaction type.

Each variant wraps the type-specific fields for that transaction kind.
Use the typed accessor methods on [`Transaction`] (e.g. `as_payment()`) to
borrow the inner fields without matching manually.

The enum is `#[non_exhaustive]` so that new XRPL amendments can be
represented by the `Unknown` catch-all without breaking existing match arms.

# Examples

```rust
use xrpl::types::TransactionType;
// Typically obtained by deserializing a Transaction from a WebSocket message.
```

```rust
pub enum TransactionType {
    AccountDelete(account::AccountDelete),
    AccountSet(account::AccountSet),
    AMMBid(amm::AMMBid),
    AMMClawback(amm::AMMClawback),
    AMMCreate(amm::AMMCreate),
    AMMDelete(amm::AMMDelete),
    AMMDeposit(amm::AMMDeposit),
    AMMVote(amm::AMMVote),
    AMMWithdraw(amm::AMMWithdraw),
    CheckCancel(payment::CheckCancel),
    CheckCash(payment::CheckCash),
    CheckCreate(payment::CheckCreate),
    Clawback(clawback::Clawback),
    CredentialAccept(credential::CredentialAccept),
    CredentialCreate(credential::CredentialCreate),
    CredentialDelete(credential::CredentialDelete),
    DepositPreauth(account::DepositPreauth),
    DIDDelete(did::DIDDelete),
    DIDSet(did::DIDSet),
    EscrowCancel(escrow::EscrowCancel),
    EscrowCreate(escrow::EscrowCreate),
    EscrowFinish(escrow::EscrowFinish),
    MPTokenAuthorize(mpt::MPTokenAuthorize),
    MPTokenIssuanceCreate(mpt::MPTokenIssuanceCreate),
    MPTokenIssuanceDestroy(mpt::MPTokenIssuanceDestroy),
    MPTokenIssuanceSet(mpt::MPTokenIssuanceSet),
    NFTokenAcceptOffer(nft::NFTokenAcceptOffer),
    NFTokenBurn(nft::NFTokenBurn),
    NFTokenCancelOffer(nft::NFTokenCancelOffer),
    NFTokenCreateOffer(nft::NFTokenCreateOffer),
    NFTokenMint(nft::NFTokenMint),
    OfferCancel(offer::OfferCancel),
    OfferCreate(offer::OfferCreate),
    OracleDelete(oracle::OracleDelete),
    OracleSet(oracle::OracleSet),
    Payment(payment::Payment),
    PaymentChannelClaim(payment_channel::PaymentChannelClaim),
    PaymentChannelCreate(payment_channel::PaymentChannelCreate),
    PaymentChannelFund(payment_channel::PaymentChannelFund),
    SetRegularKey(account::SetRegularKey),
    SignerListSet(account::SignerListSet),
    TicketCreate(account::TicketCreate),
    TrustSet(trust_set::TrustSet),
    XChainAccountCreateCommit(xchain::XChainAccountCreateCommit),
    XChainAddAccountCreateAttestation(xchain::XChainAddAccountCreateAttestation),
    XChainAddClaimAttestation(xchain::XChainAddClaimAttestation),
    XChainClaim(xchain::XChainClaim),
    XChainCommit(xchain::XChainCommit),
    XChainCreateBridge(xchain::XChainCreateBridge),
    XChainCreateClaimID(xchain::XChainCreateClaimID),
    XChainModifyBridge(xchain::XChainModifyBridge),
    Unknown {
        name: String,
        extra: serde_json::Map<String, serde_json::Value>,
    },
}
```

##### Variants

###### `AccountDelete`

Remove an account from the ledger permanently.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::AccountDelete` |  |

###### `AccountSet`

Modify account settings and flags.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::AccountSet` |  |

###### `AMMBid`

Bid on the AMM continuous auction slot for a discounted trading fee.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMBid` |  |

###### `AMMClawback`

Reclaim tokens issued via an AMM from an unauthorized holder.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMClawback` |  |

###### `AMMCreate`

Create a new AMM pool for two assets.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMCreate` |  |

###### `AMMDelete`

Delete an empty AMM pool.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMDelete` |  |

###### `AMMDeposit`

Add liquidity to an AMM pool in exchange for LP tokens.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMDeposit` |  |

###### `AMMVote`

Cast a vote for the AMM pool trading fee.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMVote` |  |

###### `AMMWithdraw`

Remove liquidity from an AMM pool by redeeming LP tokens.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `amm::AMMWithdraw` |  |

###### `CheckCancel`

Cancel an outstanding check without cashing it.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment::CheckCancel` |  |

###### `CheckCash`

Cash a check, transferring funds from the creator to the destination.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment::CheckCash` |  |

###### `CheckCreate`

Create a deferred payment check that the destination can cash later.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment::CheckCreate` |  |

###### `Clawback`

Reclaim issued tokens from a holder's trust line.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `clawback::Clawback` |  |

###### `CredentialAccept`

Accept a verifiable credential issued to the signer's account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `credential::CredentialAccept` |  |

###### `CredentialCreate`

Issue a verifiable credential to another account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `credential::CredentialCreate` |  |

###### `CredentialDelete`

Delete a verifiable credential from the ledger.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `credential::CredentialDelete` |  |

###### `DepositPreauth`

Grant or revoke deposit pre-authorization for an account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::DepositPreauth` |  |

###### `DIDDelete`

Delete a DID document from the ledger.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `did::DIDDelete` |  |

###### `DIDSet`

Create or update a DID document on the ledger.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `did::DIDSet` |  |

###### `EscrowCancel`

Cancel a time-locked or condition-locked escrow.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `escrow::EscrowCancel` |  |

###### `EscrowCreate`

Create a time-locked or condition-locked XRP escrow.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `escrow::EscrowCreate` |  |

###### `EscrowFinish`

Release funds from an escrow once conditions are met.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `escrow::EscrowFinish` |  |

###### `MPTokenAuthorize`

Authorize or un-authorize an MPT holder.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `mpt::MPTokenAuthorize` |  |

###### `MPTokenIssuanceCreate`

Create a new MPT issuance definition.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `mpt::MPTokenIssuanceCreate` |  |

###### `MPTokenIssuanceDestroy`

Destroy an MPT issuance that has no outstanding tokens.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `mpt::MPTokenIssuanceDestroy` |  |

###### `MPTokenIssuanceSet`

Update flags or properties of an MPT issuance.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `mpt::MPTokenIssuanceSet` |  |

###### `NFTokenAcceptOffer`

Accept a buy or sell offer for an NFToken.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `nft::NFTokenAcceptOffer` |  |

###### `NFTokenBurn`

Destroy an NFToken owned by the signer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `nft::NFTokenBurn` |  |

###### `NFTokenCancelOffer`

Cancel one or more NFToken buy or sell offers.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `nft::NFTokenCancelOffer` |  |

###### `NFTokenCreateOffer`

Create an offer to buy or sell an NFToken.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `nft::NFTokenCreateOffer` |  |

###### `NFTokenMint`

Mint a new NFToken into the signer's collection.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `nft::NFTokenMint` |  |

###### `OfferCancel`

Cancel an existing DEX offer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `offer::OfferCancel` |  |

###### `OfferCreate`

Place a new DEX offer to exchange one asset for another.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `offer::OfferCreate` |  |

###### `OracleDelete`

Delete a price oracle entry from the ledger.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `oracle::OracleDelete` |  |

###### `OracleSet`

Create or update a price oracle entry.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `oracle::OracleSet` |  |

###### `Payment`

Transfer XRP or issued tokens from one account to another.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment::Payment` |  |

###### `PaymentChannelClaim`

Redeem a signed claim from a payment channel.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment_channel::PaymentChannelClaim` |  |

###### `PaymentChannelCreate`

Open a new unidirectional payment channel.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment_channel::PaymentChannelCreate` |  |

###### `PaymentChannelFund`

Add more XRP to an existing payment channel.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `payment_channel::PaymentChannelFund` |  |

###### `SetRegularKey`

Assign or remove an alternate signing key for an account.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::SetRegularKey` |  |

###### `SignerListSet`

Create, replace, or delete a multi-signature signer list.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::SignerListSet` |  |

###### `TicketCreate`

Reserve one or more sequence-number tickets for future use.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `account::TicketCreate` |  |

###### `TrustSet`

Create or modify a trust line for an issued currency.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `trust_set::TrustSet` |  |

###### `XChainAccountCreateCommit`

Lock XRP on the locking chain to initiate a cross-chain account creation.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainAccountCreateCommit` |  |

###### `XChainAddAccountCreateAttestation`

Submit a signer attestation for a cross-chain account-create transfer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainAddAccountCreateAttestation` |  |

###### `XChainAddClaimAttestation`

Submit a signer attestation for a cross-chain asset transfer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainAddClaimAttestation` |  |

###### `XChainClaim`

Claim funds on the destination chain of a cross-chain transfer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainClaim` |  |

###### `XChainCommit`

Lock assets on the source chain to initiate a cross-chain transfer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainCommit` |  |

###### `XChainCreateBridge`

Register a new cross-chain bridge on the ledger.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainCreateBridge` |  |

###### `XChainCreateClaimID`

Reserve a cross-chain claim ID for an incoming transfer.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainCreateClaimID` |  |

###### `XChainModifyBridge`

Update parameters of an existing cross-chain bridge.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `xchain::XChainModifyBridge` |  |

###### `Unknown`

Catch-all for transaction types not yet modelled (e.g., new amendments).

Fields:

| Name | Type | Documentation |
|------|------|---------------|
| `name` | `String` | The raw `TransactionType` string from the wire format. |
| `extra` | `serde_json::Map<String, serde_json::Value>` | All fields from the original JSON object, preserved for inspection. |

##### Implementations

###### Trait Implementations

#### Struct `Transaction`

Common fields shared by every XRPL transaction, plus the type-specific payload.

Build a `Transaction` through the typed builder API (e.g. `PaymentBuilder`)
and sign it with a [`SigningContext`] via [`Signable::sign_with`].
For multi-signature workflows, collect [`SignerWrapper`]s and attach them
with [`Transaction::add_signatures`].

# Examples

```rust
use xrpl::types::Transaction;
// Typically obtained by deserializing a WebSocket transaction message.
```

```rust
pub struct Transaction {
    pub account: String,
    pub account_txn_id: Option<String>,
    pub fee: String,
    pub flags: Option<u32>,
    pub last_ledger_sequence: Option<u32>,
    pub memos: Option<Vec<MemoWrapper>>,
    pub sequence: u32,
    pub signers: Option<Vec<SignerWrapper>>,
    pub source_tag: Option<u32>,
    pub ticket_sequence: Option<u32>,
    pub signing_pub_key: Option<String>,
    pub txn_signature: Option<String>,
    pub hash: Option<String>,
    pub date: Option<u32>,
    pub transaction_type: TransactionType,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the account initiating the transaction. |
| `account_txn_id` | `Option<String>` | Hash of a previous transaction from this account used for mutual exclusion. |
| `fee` | `String` | Transaction cost in XRP drops (string-encoded). |
| `flags` | `Option<u32>` | Bitfield of transaction flags specific to the transaction type. |
| `last_ledger_sequence` | `Option<u32>` | The transaction is invalid and must not be applied after this ledger sequence. |
| `memos` | `Option<Vec<MemoWrapper>>` | Optional arbitrary data attached to the transaction. |
| `sequence` | `u32` | Account sequence number; must match the account's current sequence. |
| `signers` | `Option<Vec<SignerWrapper>>` | Multi-signature entries; present instead of `txn_signature` for multi-sig transactions. |
| `source_tag` | `Option<u32>` | u32 tag identifying the originating party within the sending account. |
| `ticket_sequence` | `Option<u32>` | Ticket sequence number used in place of `sequence` when tickets are enabled. |
| `signing_pub_key` | `Option<String>` | Hex-encoded public key used for single signing; empty string for multi-sig. |
| `txn_signature` | `Option<String>` | DER-encoded hex signature over the canonical serialization of this transaction. |
| `hash` | `Option<String>` | Transaction hash assigned by the ledger after validation. |
| `date` | `Option<u32>` | Ledger close time in Ripple epoch seconds (seconds since 2000-01-01T00:00:00 UTC). |
| `transaction_type` | `TransactionType` | Type-specific payload for this transaction. |

##### Implementations

###### Methods

- ```rust
  pub fn transaction_type_name(self: &Self) -> &str { /* ... */ }
  ```
  Returns the `TransactionType` field value as a string slice,

- ```rust
  pub fn as_account_delete(self: &Self) -> Option<&account::AccountDelete> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_account_set(self: &Self) -> Option<&account::AccountSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_bid(self: &Self) -> Option<&amm::AMMBid> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_clawback(self: &Self) -> Option<&amm::AMMClawback> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_create(self: &Self) -> Option<&amm::AMMCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_delete(self: &Self) -> Option<&amm::AMMDelete> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_deposit(self: &Self) -> Option<&amm::AMMDeposit> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_vote(self: &Self) -> Option<&amm::AMMVote> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_amm_withdraw(self: &Self) -> Option<&amm::AMMWithdraw> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_check_cancel(self: &Self) -> Option<&payment::CheckCancel> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_check_cash(self: &Self) -> Option<&payment::CheckCash> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_check_create(self: &Self) -> Option<&payment::CheckCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_clawback(self: &Self) -> Option<&clawback::Clawback> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_credential_accept(self: &Self) -> Option<&credential::CredentialAccept> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_credential_create(self: &Self) -> Option<&credential::CredentialCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_credential_delete(self: &Self) -> Option<&credential::CredentialDelete> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_deposit_preauth(self: &Self) -> Option<&account::DepositPreauth> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_did_delete(self: &Self) -> Option<&did::DIDDelete> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_did_set(self: &Self) -> Option<&did::DIDSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_escrow_cancel(self: &Self) -> Option<&escrow::EscrowCancel> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_escrow_create(self: &Self) -> Option<&escrow::EscrowCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_escrow_finish(self: &Self) -> Option<&escrow::EscrowFinish> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_mpt_authorize(self: &Self) -> Option<&mpt::MPTokenAuthorize> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_mpt_issuance_create(self: &Self) -> Option<&mpt::MPTokenIssuanceCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_mpt_issuance_destroy(self: &Self) -> Option<&mpt::MPTokenIssuanceDestroy> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_mpt_issuance_set(self: &Self) -> Option<&mpt::MPTokenIssuanceSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_nftoken_accept_offer(self: &Self) -> Option<&nft::NFTokenAcceptOffer> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_nftoken_burn(self: &Self) -> Option<&nft::NFTokenBurn> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_nftoken_cancel_offer(self: &Self) -> Option<&nft::NFTokenCancelOffer> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_nftoken_create_offer(self: &Self) -> Option<&nft::NFTokenCreateOffer> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_nftoken_mint(self: &Self) -> Option<&nft::NFTokenMint> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_offer_cancel(self: &Self) -> Option<&offer::OfferCancel> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_offer_create(self: &Self) -> Option<&offer::OfferCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_oracle_delete(self: &Self) -> Option<&oracle::OracleDelete> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_oracle_set(self: &Self) -> Option<&oracle::OracleSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_payment(self: &Self) -> Option<&payment::Payment> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_payment_channel_claim(self: &Self) -> Option<&payment_channel::PaymentChannelClaim> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_payment_channel_create(self: &Self) -> Option<&payment_channel::PaymentChannelCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_payment_channel_fund(self: &Self) -> Option<&payment_channel::PaymentChannelFund> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_set_regular_key(self: &Self) -> Option<&account::SetRegularKey> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_signer_list_set(self: &Self) -> Option<&account::SignerListSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_ticket_create(self: &Self) -> Option<&account::TicketCreate> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_trust_set(self: &Self) -> Option<&trust_set::TrustSet> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_account_create_commit(self: &Self) -> Option<&xchain::XChainAccountCreateCommit> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_add_account_create_attestation(self: &Self) -> Option<&xchain::XChainAddAccountCreateAttestation> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_add_claim_attestation(self: &Self) -> Option<&xchain::XChainAddClaimAttestation> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_claim(self: &Self) -> Option<&xchain::XChainClaim> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_commit(self: &Self) -> Option<&xchain::XChainCommit> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_create_bridge(self: &Self) -> Option<&xchain::XChainCreateBridge> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_create_claim_id(self: &Self) -> Option<&xchain::XChainCreateClaimID> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn as_xchain_modify_bridge(self: &Self) -> Option<&xchain::XChainModifyBridge> { /* ... */ }
  ```
  Returns the type-specific fields if this transaction matches the

- ```rust
  pub fn ticket_sequences(self: &Self) -> Option<Vec<u32>> { /* ... */ }
  ```
  For `TicketCreate` transactions, returns the sequence numbers of all allocated tickets.

- ```rust
  pub fn add_signature</* synthetic */ impl Into<Signer>: Into<Signer>>(self: &mut Self, signer: impl Into<Signer>) { /* ... */ }
  ```
  Appends a single signature and keeps the signer list sorted by account address.

- ```rust
  pub fn add_signatures<I, S>(self: &mut Self, signers: I)
where
    I: IntoIterator<Item = S>,
    S: Into<Signer> { /* ... */ }
  ```
  Attaches all signatures at once and keeps the signer list sorted by account address.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<D: serde::Deserializer<''de>>(deserializer: D) -> Result<Self, <D as >::Error> { /* ... */ }
    ```

- **MultiSignable**
  - ```rust
    fn sign_as<C: MultiSigningContext>(self: &Self, context: &C) -> Result<SignerWrapper, <C as >::Error> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<S: serde::Serializer>(self: &Self, serializer: S) -> Result<<S as >::Ok, <S as >::Error> { /* ... */ }
    ```

- **Signable**
  - ```rust
    fn sign_with<C: SigningContext>(self: &Self, context: &C) -> Result<String, <C as >::Error> { /* ... */ }
    ```

#### Struct `MemoWrapper`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Wire-format wrapper that nests a [`Memo`] under the `Memo` key in the `Memos` array.

```rust
pub struct MemoWrapper {
    pub memo: Memo,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `memo` | `Memo` | The contained memo. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `Memo`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Arbitrary data attached to a transaction, hex-encoded on the wire.

All three fields are hex strings. `MemoType` and `MemoFormat` conventionally
hold MIME types or similar descriptors (also hex-encoded).

```rust
pub struct Memo {
    pub memo_data: Option<String>,
    pub memo_format: Option<String>,
    pub memo_type: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `memo_data` | `Option<String>` | Hex-encoded memo payload. |
| `memo_format` | `Option<String>` | Hex-encoded MIME type or format descriptor for `MemoData`. |
| `memo_type` | `Option<String>` | Hex-encoded identifier for the memo's purpose or category. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(memo_data: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a memo carrying only `memo_data`. Chain [`with_format`] and

- ```rust
  pub fn with_format</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, memo_format: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the hex-encoded format descriptor.

- ```rust
  pub fn with_type</* synthetic */ impl AsRef<str>: AsRef<str>>(self: Self, memo_type: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Sets the hex-encoded type/category identifier.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `SignerWrapper`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Wire-format wrapper that nests a [`Signer`] under the `Signer` key in the `Signers` array.

```rust
pub struct SignerWrapper {
    pub signer: Signer,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signer` | `Signer` | The contained signer entry. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `Signer`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

One signature in a multi-signed transaction.

```rust
pub struct Signer {
    pub account: String,
    pub txn_signature: String,
    pub signing_pub_key: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `account` | `String` | r-address of the signing account. |
| `txn_signature` | `String` | DER-encoded hex signature produced by this signer. |
| `signing_pub_key` | `String` | Hex-encoded public key used by this signer. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl AsRef<str>: AsRef<str>>(account: impl AsRef<str>, txn_signature: impl AsRef<str>, signing_pub_key: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Creates a new `Signer` from the account, signature, and public key.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

#### Struct `SignerEntryWrapper`

Wire-format wrapper that nests a [`SignerEntry`] under the `SignerEntry` key.

```rust
pub struct SignerEntryWrapper {
    pub signer_entry: crate::types::SignerEntry,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `signer_entry` | `crate::types::SignerEntry` | The contained signer entry. |

##### Implementations

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

### Traits

#### Trait `SigningContext`

Trait for transaction signing.

Implement this on your wallet type to bridge XRPL transaction signing to a
signing crate of your choice (e.g. `ripple-keypairs`, `xrpl-mithril`).

See the [crate-level documentation](crate) for a complete example, covering
binary serialization, the `HASH_PREFIX_TRANSACTION_SIGN` prefix, and
attaching the resulting signature.

```rust
pub trait SigningContext {
    /* Associated items */
}
```

##### Required Items

###### Associated Types

- `Error`: Error type returned when signing fails.

###### Required Methods

- `sign_transaction`: Produces the final signed transaction hex string.

#### Trait `MultiSigningContext`

Trait for multi-signature transaction signing.

# Example

```rust,no_run
use anyhow::Context;
use ripple_keypairs::{PrivateKey, PublicKey};
use xrpl_mithril::codec::signing::multi_signing_data;
use xrpl::types::{Transaction, MultiSigningContext, SignerWrapper, Signer};

struct Wallet {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

impl MultiSigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_as_signer(&self, tx: &Transaction) -> Result<SignerWrapper, Self::Error> {
        let mut tx_json = serde_json::to_value(tx)
            .context("failed to convert transaction to JSON")?;
        tx_json["SigningPubKey"] = "".into();

        let map = tx_json.as_object()
            .context("transaction serialized to a non-object JSON value")?;

        // derive the 20-byte account ID from the base58 address
        let address = self.public_key.derive_address();
        let account_id: xrpl_mithril::types::AccountId =
            address.parse().context("failed to parse account address")?;

        let signing_bytes = multi_signing_data(map, account_id.as_bytes())?;
        let signature = self.private_key.sign(&signing_bytes);

        Ok(SignerWrapper {
            signer: Signer {
                account: address,
                txn_signature: signature.to_string(),
                signing_pub_key: self.public_key.to_string(),
            }
        })
    }
}
```

```rust
pub trait MultiSigningContext {
    /* Associated items */
}
```

##### Required Items

###### Associated Types

- `Error`: Error type returned when signing fails.

###### Required Methods

- `sign_as_signer`: Produce a single [`SignerWrapper`] for `tx`, to be collected with other signers.

#### Trait `Signable`

Enables single-key signing on a [`Transaction`] via `.sign_with(context)`.

```rust
pub trait Signable {
    /* Associated items */
}
```

> This trait is not object-safe and cannot be used in dynamic trait objects.

##### Required Items

###### Required Methods

- `sign_with`: Sign the transaction using `context` and return the serialized hex blob.

##### Implementations

This trait is implemented for the following types:

- `Transaction`

#### Trait `MultiSignable`

Enables multi-signature signing on a [`Transaction`] via `.sign_as(context)`.

```rust
pub trait MultiSignable {
    /* Associated items */
}
```

> This trait is not object-safe and cannot be used in dynamic trait objects.

##### Required Items

###### Required Methods

- `sign_as`: Produce a [`SignerWrapper`] from `context` for this transaction.

##### Implementations

This trait is implemented for the following types:

- `Transaction`

### Re-exports

#### Re-export `AccountDelete`

```rust
pub use account::AccountDelete;
```

#### Re-export `AccountSet`

```rust
pub use account::AccountSet;
```

#### Re-export `DepositPreauth`

```rust
pub use account::DepositPreauth;
```

#### Re-export `SetRegularKey`

```rust
pub use account::SetRegularKey;
```

#### Re-export `SignerListSet`

```rust
pub use account::SignerListSet;
```

#### Re-export `TicketCreate`

```rust
pub use account::TicketCreate;
```

#### Re-export `AMMBid`

```rust
pub use amm::AMMBid;
```

#### Re-export `AMMClawback`

```rust
pub use amm::AMMClawback;
```

#### Re-export `AMMCreate`

```rust
pub use amm::AMMCreate;
```

#### Re-export `AMMDelete`

```rust
pub use amm::AMMDelete;
```

#### Re-export `AMMDeposit`

```rust
pub use amm::AMMDeposit;
```

#### Re-export `AMMDepositFlags`

```rust
pub use amm::AMMDepositFlags;
```

#### Re-export `AMMVote`

```rust
pub use amm::AMMVote;
```

#### Re-export `AMMWithdraw`

```rust
pub use amm::AMMWithdraw;
```

#### Re-export `AMMWithdrawFlags`

```rust
pub use amm::AMMWithdrawFlags;
```

#### Re-export `Clawback`

```rust
pub use clawback::Clawback;
```

#### Re-export `CredentialAccept`

```rust
pub use credential::CredentialAccept;
```

#### Re-export `CredentialCreate`

```rust
pub use credential::CredentialCreate;
```

#### Re-export `CredentialDelete`

```rust
pub use credential::CredentialDelete;
```

#### Re-export `DIDDelete`

```rust
pub use did::DIDDelete;
```

#### Re-export `DIDSet`

```rust
pub use did::DIDSet;
```

#### Re-export `EscrowCancel`

```rust
pub use escrow::EscrowCancel;
```

#### Re-export `EscrowCreate`

```rust
pub use escrow::EscrowCreate;
```

#### Re-export `EscrowFinish`

```rust
pub use escrow::EscrowFinish;
```

#### Re-export `MPTokenAuthorize`

```rust
pub use mpt::MPTokenAuthorize;
```

#### Re-export `MPTokenAuthorizeFlags`

```rust
pub use mpt::MPTokenAuthorizeFlags;
```

#### Re-export `MPTokenIssuanceCreate`

```rust
pub use mpt::MPTokenIssuanceCreate;
```

#### Re-export `MPTokenIssuanceCreateFlags`

```rust
pub use mpt::MPTokenIssuanceCreateFlags;
```

#### Re-export `MPTokenIssuanceDestroy`

```rust
pub use mpt::MPTokenIssuanceDestroy;
```

#### Re-export `MPTokenIssuanceSet`

```rust
pub use mpt::MPTokenIssuanceSet;
```

#### Re-export `MPTokenIssuanceSetAction`

```rust
pub use mpt::MPTokenIssuanceSetAction;
```

#### Re-export `NFTokenAcceptOffer`

```rust
pub use nft::NFTokenAcceptOffer;
```

#### Re-export `NFTokenBurn`

```rust
pub use nft::NFTokenBurn;
```

#### Re-export `NFTokenCancelOffer`

```rust
pub use nft::NFTokenCancelOffer;
```

#### Re-export `NFTokenCreateOffer`

```rust
pub use nft::NFTokenCreateOffer;
```

#### Re-export `NFTokenCreateOfferFlags`

```rust
pub use nft::NFTokenCreateOfferFlags;
```

#### Re-export `NFTokenMint`

```rust
pub use nft::NFTokenMint;
```

#### Re-export `NFTokenMintFlags`

```rust
pub use nft::NFTokenMintFlags;
```

#### Re-export `OfferCancel`

```rust
pub use offer::OfferCancel;
```

#### Re-export `OfferCreate`

```rust
pub use offer::OfferCreate;
```

#### Re-export `OfferCreateFlags`

```rust
pub use offer::OfferCreateFlags;
```

#### Re-export `OracleDelete`

```rust
pub use oracle::OracleDelete;
```

#### Re-export `OracleSet`

```rust
pub use oracle::OracleSet;
```

#### Re-export `PriceData`

```rust
pub use oracle::PriceData;
```

#### Re-export `PriceDataWrapper`

```rust
pub use oracle::PriceDataWrapper;
```

#### Re-export `CheckCancel`

```rust
pub use payment::CheckCancel;
```

#### Re-export `CheckCash`

```rust
pub use payment::CheckCash;
```

#### Re-export `CheckCreate`

```rust
pub use payment::CheckCreate;
```

#### Re-export `PathStep`

```rust
pub use payment::PathStep;
```

#### Re-export `Payment`

```rust
pub use payment::Payment;
```

#### Re-export `PaymentFlag`

```rust
pub use payment::PaymentFlag;
```

#### Re-export `PaymentFlags`

```rust
pub use payment::PaymentFlags;
```

#### Re-export `PaymentChannelClaim`

```rust
pub use payment_channel::PaymentChannelClaim;
```

#### Re-export `PaymentChannelClaimAction`

```rust
pub use payment_channel::PaymentChannelClaimAction;
```

#### Re-export `PaymentChannelCreate`

```rust
pub use payment_channel::PaymentChannelCreate;
```

#### Re-export `PaymentChannelFund`

```rust
pub use payment_channel::PaymentChannelFund;
```

#### Re-export `TrustSet`

```rust
pub use trust_set::TrustSet;
```

#### Re-export `TrustSetFlags`

```rust
pub use trust_set::TrustSetFlags;
```

#### Re-export `XChainAccountCreateCommit`

```rust
pub use xchain::XChainAccountCreateCommit;
```

#### Re-export `XChainAddAccountCreateAttestation`

```rust
pub use xchain::XChainAddAccountCreateAttestation;
```

#### Re-export `XChainAddClaimAttestation`

```rust
pub use xchain::XChainAddClaimAttestation;
```

#### Re-export `XChainClaim`

```rust
pub use xchain::XChainClaim;
```

#### Re-export `XChainCommit`

```rust
pub use xchain::XChainCommit;
```

#### Re-export `XChainCreateBridge`

```rust
pub use xchain::XChainCreateBridge;
```

#### Re-export `XChainCreateClaimID`

```rust
pub use xchain::XChainCreateClaimID;
```

#### Re-export `XChainModifyBridge`

```rust
pub use xchain::XChainModifyBridge;
```

#### Re-export `XChainModifyBridgeFlags`

```rust
pub use xchain::XChainModifyBridgeFlags;
```

## Module `validation`

Address, currency-code, and amount validation helpers.

```rust
pub mod validation { /* ... */ }
```

### Types

#### Enum `ValidationError`

Errors returned by the input-validation helpers in this module.

# Examples

```rust
use xrpl::types::validation::{ValidationError, validate_address};
let err = validate_address("not-an-address").unwrap_err();
assert!(matches!(err, ValidationError::InvalidAddress(_)));
```

```rust
pub enum ValidationError {
    InvalidAddress(String),
    InvalidAmount(String),
    InvalidCurrency(String),
    InvalidMptId(String),
    InvalidInvoiceId(String),
    InvalidDomain(String),
    InvalidEmailHash(String),
    InvalidMessageKey(String),
}
```

##### Variants

###### `InvalidAddress`

The supplied r-address or X-address is structurally invalid.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidAmount`

The supplied amount value is out of range or malformed.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidCurrency`

The currency code fails length or character constraints.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidMptId`

The MPT issuance ID is not a valid 48-character hex string.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidInvoiceId`

The invoice ID is not a valid 64-character hex string (32 bytes).

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidDomain`

The domain is not valid hex.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidEmailHash`

The email hash is not a valid 32-character hex string.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

###### `InvalidMessageKey`

The message key is not valid hex.

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

##### Implementations

###### Trait Implementations

- **Display**
  - ```rust
    fn fmt(self: &Self, __formatter: &mut ::core::fmt::Formatter<''_>) -> ::core::fmt::Result { /* ... */ }
    ```

- **Error**
### Functions

#### Function `validate_address`

Checks that `address` is a syntactically valid XRPL classic or X-address.

Classic addresses start with `r` and are 25-35 alphanumeric characters.
X-addresses start with `X` (mainnet) or `T` (testnet) and are exactly 47 characters.

```rust
pub fn validate_address(address: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_currency_code`

Validates currency codes

```rust
pub fn validate_currency_code(currency: &str, xrp_allowed: bool) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_mpt_id`

Validates MPT issuance IDs (48-character hex strings)

```rust
pub fn validate_mpt_id(mpt_id: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_invoice_id`

Validates invoice IDs (64-character hex strings representing 32 bytes).

```rust
pub fn validate_invoice_id(id: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_domain`

Validates domains (hex-encoded).

```rust
pub fn validate_domain(domain: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_email_hash`

Validates email hashes (32-character hex strings representing 16 bytes).

```rust
pub fn validate_email_hash(hash: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_message_key`

Validates message keys (hex-encoded, typically 66 characters).

```rust
pub fn validate_message_key(key: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_amount_string`

Validates amount strings for issued currencies and MPTs

```rust
pub fn validate_amount_string(value: &str) -> Result<(), ValidationError> { /* ... */ }
```

#### Function `validate_amount`

Validates XRP or Issued Currency amounts.

```rust
pub fn validate_amount(amount: &super::Amount) -> Result<(), ValidationError> { /* ... */ }
```

## Module `xchain`

Cross-chain bridge type definitions.

```rust
pub mod xchain { /* ... */ }
```

### Types

#### Struct `XChainBridge`

**Attributes:**

- `Other("#[serde(rename_all = \"PascalCase\")]")`

Identifies the two door accounts and assets of a cross-chain bridge.

# Example
```rust
use xrpl::types::{Asset, xchain::XChainBridge};

let bridge = XChainBridge {
    locking_chain_door: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
    locking_chain_issue: Asset::xrp(),
    issuing_chain_door: "rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe".to_string(),
    issuing_chain_issue: Asset::xrp(),
};
```

```rust
pub struct XChainBridge {
    pub locking_chain_door: String,
    pub locking_chain_issue: super::Asset,
    pub issuing_chain_door: String,
    pub issuing_chain_issue: super::Asset,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `locking_chain_door` | `String` | r-address of the door account on the locking chain. |
| `locking_chain_issue` | `super::Asset` | Asset locked on the locking chain (XRP or issued currency). |
| `issuing_chain_door` | `String` | r-address of the door account on the issuing chain. |
| `issuing_chain_issue` | `super::Asset` | Wrapped asset minted on the issuing chain. |

##### Implementations

###### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl Into<Asset>: Into<Asset>, /* synthetic */ impl AsRef<str>: AsRef<str>, /* synthetic */ impl Into<Asset>: Into<Asset>>(locking_chain_door: impl AsRef<str>, locking_chain_issue: impl Into<Asset>, issuing_chain_door: impl AsRef<str>, issuing_chain_issue: impl Into<Asset>) -> Self { /* ... */ }
  ```
  Creates a new `XChainBridge` describing the two chains and their assets.

###### Trait Implementations

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private228::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **Serialize**
  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private228::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

### Re-exports

#### Re-export `AccountFlag`

```rust
pub use account_flag::AccountFlag;
```

#### Re-export `AccountFlags`

```rust
pub use account_flag::AccountFlags;
```

#### Re-export `HasTransactionMeta`

```rust
pub use transaction_meta::HasTransactionMeta;
```

#### Re-export `TransactionMeta`

```rust
pub use transaction_meta::TransactionMeta;
```

#### Re-export `AccountObject`

```rust
pub use account_object::AccountObject;
```

#### Re-export `Bridge`

```rust
pub use account_object::Bridge;
```

#### Re-export `Check`

```rust
pub use account_object::Check;
```

#### Re-export `Common`

```rust
pub use account_object::Common;
```

#### Re-export `Credential`

```rust
pub use account_object::Credential;
```

#### Re-export `Did`

```rust
pub use account_object::Did;
```

#### Re-export `Escrow`

```rust
pub use account_object::Escrow;
```

#### Re-export `MPToken`

```rust
pub use account_object::MPToken;
```

#### Re-export `MPTokenIssuance`

```rust
pub use account_object::MPTokenIssuance;
```

#### Re-export `NFTokenOffer`

```rust
pub use account_object::NFTokenOffer;
```

#### Re-export `NFTokenPage`

```rust
pub use account_object::NFTokenPage;
```

#### Re-export `Offer`

```rust
pub use account_object::Offer;
```

#### Re-export `Oracle`

```rust
pub use account_object::Oracle;
```

#### Re-export `PayChannel`

```rust
pub use account_object::PayChannel;
```

#### Re-export `RippleState`

```rust
pub use account_object::RippleState;
```

#### Re-export `SignerEntry`

```rust
pub use account_object::SignerEntry;
```

#### Re-export `SignerList`

```rust
pub use account_object::SignerList;
```

#### Re-export `Ticket`

```rust
pub use account_object::Ticket;
```

#### Re-export `XChainOwnedClaimID`

```rust
pub use account_object::XChainOwnedClaimID;
```

#### Re-export `XChainOwnedCreateAccountClaimID`

```rust
pub use account_object::XChainOwnedCreateAccountClaimID;
```

#### Re-export `Amount`

```rust
pub use amount::Amount;
```

#### Re-export `Asset`

```rust
pub use asset::Asset;
```

#### Re-export `XChainBridge`

```rust
pub use xchain::XChainBridge;
```

#### Re-export `amm::*`

```rust
pub use amm::*;
```

#### Re-export `builders::*`

```rust
pub use builders::*;
```

#### Re-export `transactions::*`

```rust
pub use transactions::*;
```

#### Re-export `validation::*`

```rust
pub use validation::*;
```

## Module `util`

Account utility helpers (balance, sequence, existence, flags).
Account utility helpers - thin wrappers around common `account_info` and `server_state` queries.

```rust
pub mod util { /* ... */ }
```

### Types

#### Struct `Balances`

All three XRP balances for an account in a single pair of concurrent requests.

# Example
```no_run
use xrpl::{Client, util::Balances};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let b = xrpl::util::account_balances(&client, "rAccount...").await?;
println!("total: {} | reserved: {} | available: {}", b.total, b.reserved, b.available);
# Ok(())
# }
```

```rust
pub struct Balances {
    pub total: u64,
    pub reserved: u64,
    pub available: u64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `total` | `u64` | Total XRP balance in drops. |
| `reserved` | `u64` | XRP locked in reserves (base reserve + per-object owner reserve increment) in drops. |
| `available` | `u64` | Spendable XRP in drops (`total` minus `reserved`). |

##### Implementations

###### Trait Implementations

### Functions

#### Function `next_sequence`

Next valid sequence number for single transactions or when you wait for validation between submissions.

Reads from the current open ledger so the value is immediately usable without waiting
for the previous transaction to be validated.

# Example
```no_run
use xrpl::{Client, util::next_sequence};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let seq = next_sequence(&client, "rAccount...").await?;
# Ok(())
# }
```

```rust
pub async fn next_sequence(client: &crate::Client, account: &str) -> Result<u32, crate::XrplError> { /* ... */ }
```

#### Function `next_queued_sequence`

Next sequence number for rapid multi-transaction workflows.

Uses the tx queue (`queue: true`) to find the highest queued sequence and returns
`highest + 1`. Falls back to `account_data.sequence` when nothing is queued.
This lets you submit a burst of transactions without waiting for each to be validated.

# Example
```no_run
use xrpl::{Client, util::next_queued_sequence};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let seq = next_queued_sequence(&client, "rAccount...").await?;
# Ok(())
# }
```

```rust
pub async fn next_queued_sequence(client: &crate::Client, account: &str) -> Result<u32, crate::XrplError> { /* ... */ }
```

#### Function `account_balances`

All three XRP balances for an account in a single pair of concurrent requests.

Issues `account_info` and `server_state` concurrently, then computes all three
values from the combined result. Use this when more than one balance is needed
to avoid redundant network round-trips.

# Example
```no_run
use xrpl::{Client, util::account_balances};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let b = account_balances(&client, "rAccount...").await?;
println!("total: {} | reserved: {} | available: {}", b.total, b.reserved, b.available);
# Ok(())
# }
```

```rust
pub async fn account_balances(client: &crate::Client, account: &str) -> Result<Balances, crate::XrplError> { /* ... */ }
```

#### Function `xrp_balance`

Total XRP balance in drops.

When only the raw balance is needed, this avoids the extra `server_state` request
required by [`account_balances`]. Use [`account_balances`] when reserve or
spendable amounts are also needed.

# Example
```no_run
use xrpl::{Client, util::xrp_balance};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let drops = xrp_balance(&client, "rAccount...").await?;
println!("{} drops ({} XRP)", drops, drops / 1_000_000);
# Ok(())
# }
```

```rust
pub async fn xrp_balance(client: &crate::Client, account: &str) -> Result<u64, crate::XrplError> { /* ... */ }
```

#### Function `available_balance`

Spendable XRP balance in drops - total balance minus reserves.

Delegates to [`account_balances`]. Use that directly when reserved or total
amounts are also needed.

# Example
```no_run
use xrpl::{Client, util::available_balance};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let spendable = available_balance(&client, "rAccount...").await?;
# Ok(())
# }
```

```rust
pub async fn available_balance(client: &crate::Client, account: &str) -> Result<u64, crate::XrplError> { /* ... */ }
```

#### Function `reserved_balance`

XRP locked in reserves in drops - base reserve plus per-object owner reserve increment.

Delegates to [`account_balances`]. Use that directly when total or spendable
amounts are also needed.

# Example
```no_run
use xrpl::{Client, util::reserved_balance};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let locked = reserved_balance(&client, "rAccount...").await?;
println!("{} drops locked in reserves", locked);
# Ok(())
# }
```

```rust
pub async fn reserved_balance(client: &crate::Client, account: &str) -> Result<u64, crate::XrplError> { /* ... */ }
```

#### Function `account_exists`

Whether `account` has been funded on the validated ledger.

Sending to an unfunded account requires a payment that meets the base reserve;
otherwise the transaction fails with `tecNO_DST`.

# Example
```no_run
use xrpl::{Client, util::account_exists};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
if !account_exists(&client, "rNewAccount...").await? {
    println!("Account not yet funded - payment must meet base reserve");
}
# Ok(())
# }
```

```rust
pub async fn account_exists(client: &crate::Client, account: &str) -> Result<bool, crate::XrplError> { /* ... */ }
```

#### Function `owner_count`

Number of objects the account owns on the validated ledger.

Each owned object increases the reserve requirement by the owner reserve increment.
Useful for custom reserve calculations with known or cached reserve constants.

# Example
```no_run
use xrpl::{Client, util::owner_count};

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let count = owner_count(&client, "rAccount...").await?;
# Ok(())
# }
```

```rust
pub async fn owner_count(client: &crate::Client, account: &str) -> Result<u32, crate::XrplError> { /* ... */ }
```

#### Function `account_flags`

Active account flags from the validated ledger.

# Example
```no_run
use xrpl::{Client, util::account_flags};
use xrpl::types::AccountFlag;

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = Client::new("wss://xrplcluster.com");
let flags = account_flags(&client, "rAccount...").await?;
if flags.has(AccountFlag::RequireDest) {
    println!("destination tag required");
}
# Ok(())
# }
```

```rust
pub async fn account_flags(client: &crate::Client, account: &str) -> Result<crate::types::AccountFlags, crate::XrplError> { /* ... */ }
```

## Types

### Struct `Client`

Main client for interacting with the XRP Ledger via WebSocket.
Handles connection management, requests, and subscriptions.

# Examples

## Creating a new client
```no_run
use xrpl::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    Ok(())
}
```

## Sending a request
```no_run
use xrpl::{Client, request::account_info::AccountInfoRequest};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let req = AccountInfoRequest::new("rPT1Sjq2YGrBMTttX4GZHjKu9dyfzbpAYe");
    let response = client.request(&req).await?;
    println!("Account info: {:?}", response);
    Ok(())
}
```

## Subscribing to a stream
```no_run
use xrpl::Client;
use xrpl::subscriptions::LedgerSubscription;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let sub = LedgerSubscription::new();
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;

    while let Ok(msg) = stream.recv().await {
        println!("Received: {:?}", msg);
    }
    Ok(())
}
```

## Subscribing to transactions
```no_run
use xrpl::Client;
use xrpl::subscriptions::TransactionsSubscription;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new("wss://xrplcluster.com");
    let sub = TransactionsSubscription::validated();
    let mut session = client.subscription().await?;
    let (_resp, mut stream) = session.subscribe(&sub).await?;

    while let Ok(tx) = stream.recv().await {
        println!("Transaction {}: {} ({})",
            tx.hash,
            tx.tx_json.account,
            tx.engine_result
        );
    }
    Ok(())
}
```

```rust
pub struct Client {
    pub url: String,
    // Some fields omitted
}
```

#### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `url` | `String` |  |
| *private fields* | ... | *Some fields have been omitted* |

#### Implementations

##### Methods

- ```rust
  pub fn new</* synthetic */ impl AsRef<str>: AsRef<str>>(url: impl AsRef<str>) -> Self { /* ... */ }
  ```
  Create a new client with the default configuration.

- ```rust
  pub fn with_config</* synthetic */ impl AsRef<str>: AsRef<str>>(url: impl AsRef<str>, config: ClientConfig) -> Self { /* ... */ }
  ```
  Create a new client with a custom configuration.

- ```rust
  pub async fn request<T: XrplRequest>(self: &Self, req: &T) -> Result<<T as >::Response, XrplError> { /* ... */ }
  ```
  Send a request to the XRP Ledger and return the response.

- ```rust
  pub async fn subscription(self: &Self) -> Result<SubscriptionSession<SubscriptionEvent>, XrplError> { /* ... */ }
  ```
  Opens the shared connection backing one or more subscription streams.

##### Trait Implementations

## Macros

### Macro `xrp`

**Attributes:**

- `MacroExport`

Create an XRP Amount from a float or integer value (in units of XRP).

# Example

```rust
use xrpl::xrp;
use xrpl::types::Amount;
fn amount() {
    let amount = xrp!(1.5); // 1.5 XRP
    let amount_from_str = xrp!("1.5");
}
```

```rust
pub macro_rules! xrp {
    /* macro_rules! xrp {
    ($amount:expr) => { ... };
} */
}
```

### Macro `drops`

**Attributes:**

- `MacroExport`

Create an XRP Amount from a value in drops (1 XRP = 1,000,000 drops).

# Example

```rust
use xrpl::drops;
use xrpl::types::Amount;
fn amount() {
    let amount = drops!(1_000_000); // 1 XRP
    let amount_from_str = drops!("1000000");
}
```

```rust
pub macro_rules! drops {
    /* macro_rules! drops {
    ($amount:expr) => { ... };
} */
}
```

### Macro `issued`

**Attributes:**

- `MacroExport`

Create an issued currency Amount from value, currency code, and issuer.

# Example

```rust
use xrpl::issued;
use xrpl::types::Amount;
fn amount() {
    let amount = issued!(100, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh");
}
```

```rust
pub macro_rules! issued {
    /* macro_rules! issued {
    ($value:expr, $currency:expr, $issuer:expr) => { ... };
} */
}
```

### Macro `mpt`

**Attributes:**

- `MacroExport`

Create an MPT (Multi-Purpose Token) Amount from value and issuance ID.

# Example

```rust
use xrpl::mpt;
use xrpl::types::Amount;
fn amount() {
    let amount = mpt!(1_000_000, "0000012FFD9EE5DA93AC614B4DB94D7E0FCE415CA51BED47");
}
```

```rust
pub macro_rules! mpt {
    /* macro_rules! mpt {
    ($value:expr, $mpt_issuance_id:expr) => { ... };
} */
}
```

## Re-exports

### Re-export `XrplError`

```rust
pub use error::XrplError;
```

### Re-export `ClientConfig`

```rust
pub use config::ClientConfig;
```

### Re-export `SubscriptionEvent`

```rust
pub use session::SubscriptionEvent;
```

### Re-export `SubscriptionSession`

```rust
pub use session::SubscriptionSession;
```

### Re-export `SubscriptionStream`

```rust
pub use session::SubscriptionStream;
```

