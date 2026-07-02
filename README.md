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

let req = AccountInfoRequest::new("rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1");
let response = client.request(&req).await?;
let info = response.result()?;
println!("Balance: {}", info.account_data.balance);
```

### Subscribe to ledger closes

```rust
use xrpl::SubscriptionEvent;
use xrpl::subscriptions::LedgerSubscription;

let sub = LedgerSubscription::new();
let mut handle = client.subscription().await?;
// Keep the derived stream alive so this subscription is replayed on reconnect.
let (_, _stream) = handle.subscribe(&sub).await?;
let mut count = 0;
while let Ok(event) = handle.recv().await {
    if let SubscriptionEvent::Ledger(msg) = event {
        println!("Ledger {} closed", msg.ledger_index);
        count += 1;
        if count >= 5 {
            // Close explicitly (optional, will also happen on drop)
            handle.close();
            break;
        }
    }
}
```

### Subscribe to account transactions

```rust
use anyhow::Context;
use xrpl::subscriptions::AccountTransactionsSubscription;
use xrpl::types::HasTransactionMeta;

let sub = AccountTransactionsSubscription::proposed(
    ["rU6K7V3Po4snVhBBaU29sesqs2qTQJWDw1"],
).context("Failed to create proposed transactions subscription")?;
let mut handle = client.subscription().await?;
let (_, mut stream) = handle.subscribe(&sub).await?;

while let Ok(tx) = stream.recv().await {
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
            let map = tx_json.as_object().context("Expected a JSON object")?;
            let mut buf = Vec::new();
            serialize_json_object(map, &mut buf, true)?;
            buf
        };

        let mut signing_bytes = Vec::with_capacity(4 + buf.len());
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&buf);
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let map = tx_json.as_object().context("Expected a JSON object")?;
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
    wallet.public_key.derive_address(),
    destination_address,
    xrp!(1.99),
)
.fill(&client)
.await?
.build()?;

let submit_req = SubmitRequestBuilder::new(&payment, &wallet).build()?;
let response = client.request(&submit_req).await?;
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
use xrpl::{xrp, drops, issued, mpt};

xrp!(1.99)   // 1.99 XRP → stored as drops string
drops!(12)   // 12 drops
```

**Issued currencies (IOU)** — the legacy token model.

```rust
issued!(100.0, "USD", "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh")
```

**Multi-Purpose Tokens (MPT)** — the modern token model.

```rust
// 100.00 tokens at AssetScale 2 → pass 10000
mpt!(10000, "0000000124B5A7AE55A3019B1C7B38FBA04BEF0CEF2D6F48")
```

### NFTs

Mint, list, and trade NFTs using the full `NFToken*` builder suite.

```rust
use xrpl::types::builders::{NFTokenMintBuilder, NFTokenCreateOfferBuilder, NFTokenAcceptOfferBuilder};
use xrpl::request::{account_nfts::AccountNftsRequest, nft_sell_offers::NftSellOffersRequest};
use xrpl::{xrp};

// Mint — taxon groups tokens into collections; URI is hex-encoded
let mint = NFTokenMintBuilder::new(account, 42)
    .with_transfer_fee(5000)          // 5% royalty (units: 1/100,000 of a percent)
    .with_uri("68747470733a2f2f...")  // hex-encoded metadata URI
    .fill(&client).await?
    .build()?;

// Create a sell offer
let offer = NFTokenCreateOfferBuilder::new(account, nftoken_id, xrp!(10))
    .with_destination(buyer_address)  // optional: restrict to one buyer
    .fill(&client).await?
    .build()?;

// Accept an offer (direct sale or brokered)
let accept = NFTokenAcceptOfferBuilder::new(account)
    .with_nftoken_sell_offer(offer_id)
    .fill(&client).await?
    .build()?;

// Query NFTs owned by an account
let nfts = client.request(&AccountNftsRequest::new(account)).await?;

// Query open sell offers for a token
let offers = client.request(&NftSellOffersRequest::new(nftoken_id)).await?;
```

Transfer fees are enforced at the protocol level. Check `tfTransferable` on the
token before creating offers — non-transferable NFTs cannot be sold.

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
