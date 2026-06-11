use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{
    validation::validate_address, transactions::nft::NFTokenMint, Amount,
    TransactionType,
};

/// Builder for XRPL NFT mint (NFTokenMint) transactions.
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// use xrpl::{Client, types::builders::NFTokenMintBuilder};
/// let client = Client::new("wss://xrplcluster.com");
/// let tx = NFTokenMintBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 0)
///     .with_transfer_fee(5000)
///     .with_uri("68747470733a2f2f6578616d706c652e636f6d2f6e6674")
///     .fill(&client)
///     .await?
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub type NFTokenMintBuilder = TransactionBuilder<NFTokenMint>;

impl NFTokenMintBuilder {
    /// Creates a new `NFTokenMintBuilder` for the given taxon (collection identifier).
    pub fn new(account: impl AsRef<str>, nftoken_taxon: u32) -> Self {
        Self::init(
            account,
            0,
            Amount::default(),
            NFTokenMint {
                nftoken_taxon,
                issuer: None,
                transfer_fee: None,
                uri: None,
            },
        )
    }

    /// Sets the issuer account when minting on behalf of another account.
    pub fn with_issuer(mut self, issuer: impl AsRef<str>) -> Self {
        self.transaction_type.issuer = Some(issuer.as_ref().to_string());
        self
    }

    /// Sets the royalty fee in units of 1/100,000 of a percent (0–50000).
    pub fn with_transfer_fee(mut self, transfer_fee: u16) -> Self {
        self.transaction_type.transfer_fee = Some(transfer_fee);
        self
    }

    /// Sets the hex-encoded URI pointing to the token's metadata.
    pub fn with_uri(mut self, uri: impl AsRef<str>) -> Self {
        self.transaction_type.uri = Some(uri.as_ref().to_string());
        self
    }
}

impl TransactionTypeBuilder for NFTokenMint {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(transfer_fee) = self.transfer_fee
            && transfer_fee > 50_000u16
        {
            return Err(
                crate::types::validation::ValidationError::InvalidAmount(
                    "transfer_fee cannot exceed 50000 (50%)".to_string(),
                )
                .into(),
            );
        }
        if let Some(uri) = &self.uri {
            if uri.is_empty() {
                return Err(
                    crate::types::validation::ValidationError::InvalidAmount(
                        "URI cannot be empty if provided".to_string(),
                    )
                    .into(),
                );
            }
            if uri.len() > 512 {
                return Err(
                    crate::types::validation::ValidationError::InvalidAmount(
                        "URI cannot exceed 512 characters".to_string(),
                    )
                    .into(),
                );
            }
        }
        if let Some(issuer) = &self.issuer {
            validate_address(issuer)?;
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        self.validate()?;
        Ok(TransactionType::NFTokenMint(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nftoken_mint_basic() {
        let mint =
            NFTokenMintBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 0)
                .build()
                .expect("Should build valid NFT mint");

        if let TransactionType::NFTokenMint(NFTokenMint {
            nftoken_taxon, ..
        }) = mint.transaction_type
        {
            assert_eq!(nftoken_taxon, 0);
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_with_transfer_fee() {
        let mint =
            NFTokenMintBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 0)
                .with_transfer_fee(5000)
                .build()
                .expect("Should build valid NFT mint with transfer fee");

        if let TransactionType::NFTokenMint(NFTokenMint {
            transfer_fee, ..
        }) = mint.transaction_type
        {
            assert_eq!(transfer_fee, Some(5000));
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_transfer_fee_too_high() {
        let result =
            NFTokenMintBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 0)
                .with_transfer_fee(50_001)
                .build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }

    #[test]
    fn test_nftoken_mint_with_uri() {
        let mint =
            NFTokenMintBuilder::new("rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh", 0)
                .with_uri("68747470733a2f2f6578616d706c652e636f6d2f6e6674")
                .build()
                .expect("Should build valid NFT mint with URI");

        if let TransactionType::NFTokenMint(NFTokenMint { uri, .. }) =
            mint.transaction_type
        {
            assert!(uri.is_some());
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_invalid_account() {
        let result = NFTokenMintBuilder::new("not_an_address", 0).build();

        assert!(matches!(result, Err(BuildError::Validation(_))));
    }
}
