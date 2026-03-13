#![allow(dead_code)]

use ripple_keypairs::Seed;
use xrpl_mithril_codec::serializer;
use xrpl_mithril_codec::signing::HASH_PREFIX_TRANSACTION_SIGN;
use xrpl::types::SigningContext;

pub fn server_url() -> String {
    std::env::var("SERVER_URL")
        .unwrap_or_else(|_| "wss://s.altnet.rippletest.net:51233".to_string())
}

pub fn test_seed() -> String {
    std::env::var("TEST_SEED")
        .expect("TEST_SEED environment variable must be set for tests")
}

pub fn test_seed_2() -> String {
    std::env::var("TEST_SEED_2")
        .expect("TEST_SEED_2 environment variable must be set for tests")
}

pub fn sender_address() -> String {
    let seed_str = test_seed();
    let seed: Seed = seed_str.parse().expect("Failed to parse test seed");
    let (_, public_key) =
        seed.derive_keypair().expect("Failed to derive keypair from test seed");
    public_key.derive_address()
}

pub fn receiver_address() -> String {
    let seed_str = test_seed_2();
    let seed: Seed = seed_str.parse().expect("Failed to parse test seed 2");
    let (_, public_key) = seed
        .derive_keypair()
        .expect("Failed to derive keypair from test seed 2");
    public_key.derive_address()
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
        let mut tx_json =
            serde_json::to_value(tx).expect("Failed to convert to json");
        tx_json["SigningPubKey"] = self.public_key.to_string().into();

        let map =
            tx_json.as_object().expect("Transaction should be JSON object");

        // Serialize for signing (without signature fields)
        let mut signing_buf = Vec::new();
        serializer::serialize_json_object(map, &mut signing_buf, true)?; // true = for signing
        let tx_hex = hex::encode(signing_buf);

        // Standard XRPL signing: STX prefix + serialized transaction hash
        let mut signing_bytes = Vec::with_capacity(4 + tx_hex.len() / 2);
        signing_bytes.extend_from_slice(&HASH_PREFIX_TRANSACTION_SIGN);
        signing_bytes.extend_from_slice(&hex::decode(tx_hex)?);
        let signature = self.private_key.sign(&signing_bytes);
        tx_json["TxnSignature"] = signature.to_string().into();

        // Serialize final signed transaction
        let final_map =
            tx_json.as_object().expect("Transaction should be JSON object");
        let mut final_buf = Vec::new();
        serializer::serialize_json_object(final_map, &mut final_buf, false)?; // false = not for signing

        Ok(hex::encode(final_buf).to_uppercase())
    }
}
