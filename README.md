# xrpl-ws

Lightweight async WebSocket client for the XRP Ledger. Supports requests,
subscriptions, and automatic reconnection. Reconnects automatically on network
interruption. Active subscriptions are replayed after reconnect.

## Installation

```toml
[dependencies]
xrpl-ws = "0.1"
```

Imports as `xrpl`:

```rust
use xrpl::{Client, types::Amount, types::builders::PaymentBuilder};
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
    .subscription_channel_size(128);

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

let (initial, mut handle) = client.subscribe(LedgerSubscription::new()).await?;
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

let (_, mut handle) = client
    .subscribe(AccountTransactionsSubscription::proposed(
        vec!["rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".into()],
    ).map_err(|e| anyhow!(e))?)
    .await?;

while let Ok(msg) = handle.recv().await {
    if msg.validated {
        println!("Transaction is validated: {}", msg.hash);
        
        if let TransactionType::Payment { amount, deliver_max, .. } =
         msg.tx_json.transaction_type {
            let payment_amount = amount
                .or(deliver_max)
                .unwrap_or_else(|| Amount::Xrpl("0".to_string()));

            eprintln!("Transaction amount: {}", payment_amount);
        }
    } else {
        println!("Transaction not yet validated: {}", msg.hash);
    }
}
```

### Sign and submit a transaction

Signing requires implementing the `SigningContext` trait for your wallet type.
The process follows the XRPL signing protocol: serialize the transaction with
a 4-byte prefix (`HASH_PREFIX_TRANSACTION_SIGN`), sign the bytes, attach the signature, then
serialize the final blob for submission.

```rust
use anyhow::anyhow;
use ripple_keypairs::{PrivateKey, PublicKey};
use xrpl_mithril_codec::serializer::serialize_tx;
use xrpl_mithril_codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
use xrpl::types::{Transaction, SigningContext};

struct Wallet {
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl SigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_transaction(&self, tx: &Transaction) -> Result<String, Self::Error> {
        // Attach the public key to the transaction JSON
        let mut tx_json = serde_json::to_value(tx)
            .expect("Failed to convert transaction to json");
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        // Serialize to XRPL binary format for signing (canonical = true)
        let tx_hex = serialize_tx(serde_json::to_string(&tx_json)?, true)
            .ok_or_else(|| anyhow!("Failed to serialize transaction for signing"))?;

        // Prepend the XRPL signing prefix and sign
        let mut signing_bytes = Vec::with_capacity(4 + tx_hex.len() / 2);
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&hex::decode(tx_hex)?);
        let signature = self.private_key.sign(&signing_bytes);

        // Attach signature and serialize the final blob (canonical = false)
        tx_json["TxnSignature"] = signature.to_string().into();
        let tx_signed = serialize_tx(serde_json::to_string(&tx_json)?, false)
            .ok_or_else(|| anyhow!("Failed to serialize signed transaction"))?;

        Ok(tx_signed)
    }
}
```

Build the transaction, sign it, and submit:

```rust
use xrpl::types::builders::PaymentBuilder;
use xrpl::request::submit::SubmitRequest;
use xrpl::{xrp, drops};

let wallet = Wallet { public_key, private_key };

let payment = PaymentBuilder::new(
    wallet.public_key.derive_address().into(),
    destination_address,
    sequence,
    drops!(10), // fee
    xrp!(1.99), // amount
)
.build()?;

let tx_blob = payment.sign_with(&wallet)?;
let response = client.request(SubmitRequest { tx_blob, fail_hard: None }).await?;
let result = response.result()?;
assert_eq!(result.engine_result, "tesSUCCESS");
```

See [tests/transaction.rs](tests/transaction.rs) for a complete example
including key derivation, transaction building, signing, and submission.

### Amount helpers

```rust
use xrpl::{xrp, drops, issued};

xrp!(1.99)                                 // 1.99 XRP
drops!(12)                                 // 12 drops
issued!(100.0, "USD", "rIssuerAddress...") // issued currency
```

### Running integration tests

```bash
export TEST_SEED="sEd.."
export TEST_SEED_2="sEd.."

cargo test -- --ignored --nocapture
```
