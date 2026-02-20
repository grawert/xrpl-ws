use super::{BuildError, TransactionBuilder, TransactionTypeBuilder};
use crate::types::{Amount, TransactionType};

pub struct NFTokenMint {
    pub nftoken_taxon: u32,
    pub issuer: Option<String>,
    pub transfer_fee: Option<u32>,
    pub uri: Option<String>,
}

pub type NFTokenMintBuilder = TransactionBuilder<NFTokenMint>;

/// Create a new NFT mint transaction.
///
/// # Example
/// ```no_run
/// use xrpl::types::Amount;
/// use xrpl::types::builders::NFTokenMintBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mint = NFTokenMintBuilder::new(
///     "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
///     1,
///     Amount::from(10u64),
///     0,
/// )
/// .with_transfer_fee(5000)
/// .with_uri("68747470733a2f2f6578616d706c652e636f6d2f6e6674".to_string())
/// .build()?;
/// # Ok(())
/// # }
/// ```
impl NFTokenMintBuilder {
    pub fn new(
        account: String,
        sequence: u32,
        fee: Amount,
        nftoken_taxon: u32,
    ) -> Self {
        Self::init(
            account,
            sequence,
            fee,
            NFTokenMint {
                nftoken_taxon,
                issuer: None,
                transfer_fee: None,
                uri: None,
            },
        )
    }

    pub fn with_issuer(mut self, issuer: String) -> Self {
        self.transaction_type.issuer = Some(issuer);
        self
    }

    pub fn with_transfer_fee(mut self, transfer_fee: u32) -> Self {
        self.transaction_type.transfer_fee = Some(transfer_fee);
        self
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.transaction_type.uri = Some(uri);
        self
    }
}

impl TransactionTypeBuilder for NFTokenMint {
    type TransactionType = TransactionType;

    fn validate(&self) -> Result<(), BuildError> {
        if let Some(transfer_fee) = self.transfer_fee {
            if transfer_fee > 50_000 {
                return Err(BuildError::InvalidField(
                    "transfer_fee cannot exceed 50000 (50%)".to_string(),
                ));
            }
        }
        if let Some(uri) = &self.uri {
            if uri.is_empty() {
                return Err(BuildError::InvalidField(
                    "URI cannot be empty if provided".to_string(),
                ));
            }
            if uri.len() > 512 {
                return Err(BuildError::InvalidField(
                    "URI cannot exceed 512 characters".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn build_transaction_type(
        self,
    ) -> Result<Self::TransactionType, BuildError> {
        Ok(TransactionType::NFTokenMint {
            nftoken_taxon: self.nftoken_taxon,
            issuer: self.issuer,
            transfer_fee: self.transfer_fee,
            uri: self.uri,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEQUENCE: u32 = 1;

    #[test]
    fn test_nftoken_mint_basic() {
        let mint = NFTokenMintBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            0,
        )
        .build()
        .expect("Should build valid NFT mint");

        if let TransactionType::NFTokenMint { nftoken_taxon, .. } =
            mint.transaction_type
        {
            assert_eq!(nftoken_taxon, 0);
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_with_transfer_fee() {
        let mint = NFTokenMintBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            0,
        )
        .with_transfer_fee(5000)
        .build()
        .expect("Should build valid NFT mint with transfer fee");

        if let TransactionType::NFTokenMint { transfer_fee, .. } =
            mint.transaction_type
        {
            assert_eq!(transfer_fee, Some(5000));
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_transfer_fee_too_high() {
        let result = NFTokenMintBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            0,
        )
        .with_transfer_fee(50_001)
        .build();

        assert!(matches!(result, Err(BuildError::InvalidField(_))));
    }

    #[test]
    fn test_nftoken_mint_with_uri() {
        let mint = NFTokenMintBuilder::new(
            "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
            SEQUENCE,
            drops!(10),
            0,
        )
        .with_uri("68747470733a2f2f6578616d706c652e636f6d2f6e6674".to_string())
        .build()
        .expect("Should build valid NFT mint with URI");

        if let TransactionType::NFTokenMint { uri, .. } = mint.transaction_type
        {
            assert!(uri.is_some());
        } else {
            panic!("Expected NFTokenMint transaction type");
        }
    }

    #[test]
    fn test_nftoken_mint_invalid_account() {
        let result = NFTokenMintBuilder::new(
            "not_an_address".to_string(),
            SEQUENCE,
            drops!(10),
            0,
        )
        .build();

        assert!(matches!(result, Err(BuildError::InvalidField(_))));
    }
}
