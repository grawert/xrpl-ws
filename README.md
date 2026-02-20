# xrpl-ws

Lightweight async WebSocket client for the XRP Ledger. Supports requests,
subscriptions, and automatic reconnection with exponential backoff.

## Installation

```toml
[dependencies]
xrpl-ws = "0.1"
```

Imports as `xrpl`:

```rust
use xrpl::{XrplClient, types::Amount, types::builders::PaymentBuilder};
```

## Usage

### Connect

```rust
let client = XrplClient::new("wss://xrplcluster.com").await?;
```

Reconnects automatically on network interruption. Active subscriptions are
replayed after reconnect.

### Query account info

```rust
let response = client.request(AccountInfoRequest {
    account: "rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".into(),
    ..Default::default()
}).await?;

let info = response.result()?;
println!("Balance: {}", info.account_data.balance);
```

### Subscribe to ledger closes

```rust
let (initial, mut rx) = client.subscribe(LedgerSubscription).await?;

while let Ok(msg) = rx.recv().await {
    println!("Ledger {} closed", msg.ledger_index);
}
```

### Subscribe to account transactions

```rust
let (_, mut rx) = client.subscribe(AccountTransactionsSubscription {
    accounts: vec!["rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1".into()],
}).await?;

while let Ok(tx) = rx.recv().await {
    println!("Transaction: {:?}", tx);
}
```

### Sign and submit a transaction

Signing requires implementing the `SigningContext` trait for your wallet type.
The process follows the XRPL signing protocol: serialize the transaction with
a 4-byte prefix (`53545800`), sign the bytes, attach the signature, then
serialize the final blob for submission.

```rust
use anyhow::anyhow;
use ripple_keypairs::{PrivateKey, PublicKey};
use rippled_binary_codec::serialize::serialize_tx;
use xrpl::types::{Transaction, SigningContext};

const STX_PREFIX: &str = "53545800";

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
        let signing_bytes = hex::decode(format!("{}{}", STX_PREFIX, tx_hex))?;
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
xrp!(1.99)                                 // 1.99 XRP
drops!(12)                                 // 12 drops
issued!(100.0, "USD", "rIssuerAddress...") // issued currency
```
