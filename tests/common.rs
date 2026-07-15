#![allow(dead_code, unused_imports)]

use anyhow::Context;
use ripple_keypairs::Seed;
use xrpl_mithril::codec::serializer;
use xrpl_mithril::codec::signing::{
    HASH_PREFIX_TRANSACTION_SIGN, multi_signing_data,
};
use xrpl::request::submit::SubmitResponse;
use xrpl::types::{MultiSigningContext, Signer, SignerWrapper, SigningContext};

pub use xrpl::time::ripple_now;

pub fn assert_accepted(result: &SubmitResponse, context: &str) {
    let code = &result.engine_result;
    assert!(
        !code.starts_with("tem"),
        "{context}: transaction malformed ({code}) - amendment may not be active on this network"
    );
    assert!(
        code.starts_with("tes"),
        "{context}: transaction failed - engine_result: {code}"
    );
}

pub fn server_url() -> String {
    std::env::var("SERVER_URL")
        .unwrap_or_else(|_| "wss://s.altnet.rippletest.net:51233".to_string())
}

/// Returns the seed for the n-th funded test account, read from the
/// `TEST_SEED_<n>` environment variable. `n` is 1-based, matching the
/// numbering used by `scripts/fund_test_accounts.py`.
pub fn test_seed(n: usize) -> String {
    let var = format!("TEST_SEED_{n}");
    std::env::var(&var).unwrap_or_else(|_| {
        panic!("{var} environment variable must be set for tests")
    })
}

pub fn sender_address() -> String {
    let seed_str = test_seed(1);
    let seed: Seed = seed_str.parse().expect("Failed to parse TEST_SEED_1");
    let (_, public_key) = seed
        .derive_keypair()
        .expect("Failed to derive keypair from TEST_SEED_1");
    public_key.derive_address()
}

pub fn receiver_address() -> String {
    let seed_str = test_seed(2);
    let seed: Seed = seed_str.parse().expect("Failed to parse TEST_SEED_2");
    let (_, public_key) = seed
        .derive_keypair()
        .expect("Failed to derive keypair from TEST_SEED_2");
    public_key.derive_address()
}

/// Returns the fully-derived wallet (public + private key) for the primary
/// funded test account (`TEST_SEED_1`) - the sender in most integration tests.
pub fn sender_wallet() -> Wallet {
    let seed_str = test_seed(1);
    let seed: Seed = seed_str.parse().expect("Failed to parse TEST_SEED_1");
    let (private_key, public_key) = seed
        .derive_keypair()
        .expect("Failed to derive keypair from TEST_SEED_1");
    Wallet { public_key, private_key }
}

pub struct Wallet {
    pub public_key: ripple_keypairs::PublicKey,
    pub private_key: ripple_keypairs::PrivateKey,
}

impl SigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_transaction(
        &self,
        tx: &xrpl::types::Transaction,
    ) -> anyhow::Result<String, Self::Error> {
        let mut tx_json = serde_json::to_value(tx)
            .context("Failed to serialize transaction to JSON")?;
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let buf = {
            let map = tx_json
                .as_object()
                .context("Transaction JSON is not an object")?;
            let mut buf = Vec::new();
            serializer::serialize_json_object(map, &mut buf, true)?;
            buf
        };

        let mut signing_bytes = Vec::with_capacity(4 + buf.len());
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&buf);

        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        let map =
            tx_json.as_object().context("Transaction JSON is not an object")?;
        let mut buf = Vec::new();
        serializer::serialize_json_object(map, &mut buf, false)?;

        Ok(hex::encode(buf).to_uppercase())
    }
}

impl MultiSigningContext for Wallet {
    type Error = anyhow::Error;

    fn sign_as_signer(
        &self,
        tx: &xrpl::types::Transaction,
    ) -> anyhow::Result<SignerWrapper, Self::Error> {
        let mut tx_json = serde_json::to_value(tx)
            .context("Failed to serialize transaction to JSON")?;
        tx_json["SigningPubKey"] = "".into();

        let address = self.public_key.derive_address();
        let account_id: xrpl_mithril::types::AccountId =
            address.parse().context("Failed to parse account address")?;

        let signing_bytes = {
            let map = tx_json
                .as_object()
                .context("Transaction JSON is not an object")?;
            multi_signing_data(map, account_id.as_bytes())?
        };

        let signature = self.private_key.sign(&signing_bytes);

        Ok(SignerWrapper {
            signer: Signer {
                account: address,
                txn_signature: signature.to_string(),
                signing_pub_key: self.public_key.to_string(),
            },
        })
    }
}
