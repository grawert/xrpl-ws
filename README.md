# xrpl-ws

Lightweight async WebSocket client for the XRP Ledger. Supports requests,
subscriptions, and automatic reconnection. Transaction signing and serialization
is delegated to external libraries.

## Installation

```toml
[dependencies]
xrpl-ws = "0.1"
```

Imports as `xrpl`:

```rust
use xrpl::{Client, types::Amount, types::builders::PaymentBuilder, types::builders::SubmitRequestBuilder};
```

## Usage

### Connect

```rust
let client = Client::new("wss://xrplcluster.com");
```

### Configuration

For custom configuration, use `with_config()`:

```rust
use xrpl::{Client, ClientConfig};

let config = ClientConfig::new()
    .with_request_timeout_secs(60)
    .with_subscription_channel_size(128);

let client = Client::with_config("wss://xrplcluster.com", config);
```

### Query account info

```rust
use xrpl::request::account_info::AccountInfoRequest;

let response = client.request(AccountInfoRequest {
    account: "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".into(),
    ..Default::default()
}).await?;

let info = response.result()?;
println!("Balance: {}", info.account_data.balance);
```

### Subscribe to ledger closes

```rust
use xrpl::subscriptions::LedgerSubscription;

let (_, mut handle) = client.subscribe(LedgerSubscription::new()).await?;
let mut count = 0;
while let Ok(msg) = handle.recv().await {
    println!("Ledger {} closed", msg.ledger_index);
    count += 1;
    if count >= 5 {
        // Close explicitly (optional, will also happen on drop)
        handle.close();
        break;
    }
}
```

### Subscribe to account transactions

```rust
use anyhow::anyhow;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::HasTransactionMeta;

let (_, mut handle) = client
    .subscribe(AccountTransactionsSubscription::proposed(
        vec!["rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".into()],
    ).map_err(|e| anyhow!(e))?)
    .await?;

while let Ok(tx) = handle.recv().await {
    if tx.validated {
        println!("Transaction is validated: {}", tx.hash);

        if let Some(amount) = tx.delivered_amount() {
            eprintln!("Transaction amount: {}", amount);
        }
    } else {
        println!("Transaction not yet validated: {}", tx.hash);
    }
}
```

### Sign and submit a transaction

Signing requires implementing the `SigningContext` trait for your wallet type.
The process follows the XRPL signing protocol: serialize the transaction to
binary (excluding the signature fields), prepend `HASH_PREFIX_TRANSACTION_SIGN`
(the "STX" prefix), sign the bytes, attach the signature, then serialize the
final blob for submission.

```rust
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
        let mut tx_json = serde_json::to_value(tx)
            .expect("Failed to convert transaction to json");
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let buf = {
            let map = tx_json.as_object().expect("Transaction should be JSON object");
            let mut buf = Vec::new();
            serialize_json_object(map, &mut buf, true)?;
            buf
        };

        let mut signing_bytes = Vec::with_capacity(4 + buf.len());
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&buf);
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let map = tx_json.as_object().expect("Transaction should be JSON object");
        let mut buf = Vec::new();
        serialize_json_object(map, &mut buf, false)?;

        Ok(hex::encode(buf).to_uppercase())
    }
}
```

Build the transaction, sign it, and submit:

```rust
use xrpl::{Client, xrp};
use xrpl::types::builders::{PaymentBuilder, SubmitRequestBuilder};

let client = Client::new("wss://xrplcluster.com");
let wallet = Wallet { public_key, private_key };

let payment = PaymentBuilder::new(
    wallet.public_key.derive_address().into(),
    destination_address,
)
.with_amount(xrp!(1.99))
.fill(&client)
.await?
.build()?;

let response = client.request(
    SubmitRequestBuilder::new(&payment, &wallet).build()?
).await?;
let result = response.result()?;
assert_eq!(result.engine_result, "tesSUCCESS");
```

See [tests/transaction.rs](tests/transaction.rs) for a complete example
including key derivation, transaction building, signing, and submission.

### Time helpers

XRPL timestamps (used in `Expiration`, `FinishAfter`, `CancelAfter`) are seconds
since the **Ripple Epoch** (2000-01-01 UTC). Passing a UNIX timestamp directly
sets the time ~30 years in the future.

```rust
use xrpl::time::{ripple_now, unix_to_ripple, ripple_to_unix};

let expiry = ripple_now() + 3600; // 1 hour from now

// Convert to/from UNIX time
let unix: u64 = 1_000_000_000;
let ripple: u32 = unix_to_ripple(unix);
assert_eq!(ripple_to_unix(ripple), unix);
```

### Amount helpers

```rust
use xrpl::{xrp, drops, issued};
use xrpl::types::Amount;

xrp!(1.99) // 1.99 XRP → drops string
drops!(12) // 12 drops
issued!(100.0, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh") // issued currency

// MPT amount (integer, uses the 48-char issuance ID from MPTokenIssuanceCreate)
// 100.00 (AssetScale 2)
Amount::mpt("10000", "0000000124B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48")?
```

### Asset helpers

`Asset` identifies a tradable asset without a quantity — use it for AMM pool
sides, order book entries, and trust-line targets.

```rust
use xrpl::types::Asset;

let xrp   = Asset::xrp();
let usd   = Asset::token("USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")?;
let mpt   = Asset::mpt("0000000124B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48")?;

// Pair with a value to get an Amount
// Amount::IssuedCurrency { value: "100", ... }
let amount = usd.amount_with("100")?;
```

### Running unit tests

```bash
cargo test --package xrpl-ws --lib -- --nocapture
```

### Running integration tests

```bash
export TEST_SEED_1="sEd.."
export TEST_SEED_2="sEd.."
export TEST_SEED_3="sEd.."

cargo test --package xrpl-ws --test '*' -- --nocapture
```
