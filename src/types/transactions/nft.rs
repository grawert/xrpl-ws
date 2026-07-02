use std::ops::BitOr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::types::Amount;

/// Transaction flags for [`NFTokenMint`].
///
/// ```rust
/// use xrpl::types::NFTokenMintFlags as Flags;
///
/// let flags = Flags::BURNABLE | Flags::TRANSFERABLE;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NFTokenMintFlags(pub u32);

impl NFTokenMintFlags {
    /// Issuer or its `MintedNFToken` account can burn the token regardless of ownership.
    pub const BURNABLE: Self = Self(0x00000001);
    /// Token can only be bought or sold for XRP, not issued currencies.
    pub const ONLY_XRP: Self = Self(0x00000002);
    /// Automatically creates a trust line when transferring to an account with no trust line.
    pub const TRUST_LINE: Self = Self(0x00000004);
    /// Token can be transferred between accounts (required for secondary sales).
    pub const TRANSFERABLE: Self = Self(0x00000008);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl BitOr for NFTokenMintFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl From<u32> for NFTokenMintFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<NFTokenMintFlags> for u32 {
    fn from(f: NFTokenMintFlags) -> u32 {
        f.0
    }
}

/// Transaction flags for [`NFTokenCreateOffer`].
///
/// ```rust
/// use xrpl::types::NFTokenCreateOfferFlags;
///
/// let flags = NFTokenCreateOfferFlags::SELL;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NFTokenCreateOfferFlags(pub u32);

impl NFTokenCreateOfferFlags {
    /// Creates a sell offer (token flows from submitter to buyer).
    /// Omit to create a buy offer instead.
    pub const SELL: Self = Self(0x00000001);

    /// Returns `true` if the given flag is set in this bitmask.
    pub fn has(self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }
}

impl From<u32> for NFTokenCreateOfferFlags {
    fn from(v: u32) -> Self {
        Self(v)
    }
}

impl From<NFTokenCreateOfferFlags> for u32 {
    fn from(f: NFTokenCreateOfferFlags) -> u32 {
        f.0
    }
}

/// Completes an NFT trade by accepting an existing buy or sell offer.
///
/// In brokered mode, supply both a sell and a buy offer; the difference minus
/// `NFTokenBrokerFee` goes to the broker.
///
/// ```rust
/// use xrpl::types::transactions::nft::NFTokenAcceptOffer;
/// let tx = NFTokenAcceptOffer {
///     nftoken_sell_offer: Some("offer_id_hex".to_string()),
///     nftoken_buy_offer: None,
///     nftoken_broker_fee: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenAcceptOffer {
    /// Ledger object ID of the NFT sell offer to accept.
    #[serde(rename = "NFTokenSellOffer")]
    pub nftoken_sell_offer: Option<String>,
    /// Ledger object ID of the NFT buy offer to accept.
    #[serde(rename = "NFTokenBuyOffer")]
    pub nftoken_buy_offer: Option<String>,
    /// Fee retained by the broker in brokered-mode transactions.
    #[serde(rename = "NFTokenBrokerFee")]
    pub nftoken_broker_fee: Option<Amount>,
}

/// Permanently destroys an NFToken, removing it from the ledger.
///
/// The submitter must be the token owner, or the issuer if the token was minted with
/// the `tfBurnable` flag. Once burned, the token ID cannot be reused.
///
/// ```rust
/// use xrpl::types::transactions::nft::NFTokenBurn;
/// let tx = NFTokenBurn {
///     nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
///     owner: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenBurn {
    /// The 256-bit identifier of the NFToken to burn.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: String,
    /// Current owner, required when the issuer (not the owner) is submitting the burn.
    pub owner: Option<String>,
}

/// Cancels one or more open NFT buy or sell offers.
///
/// Multiple offer IDs can be cancelled in a single transaction. The submitter must be
/// either the offer creator or the NFT issuer (for offers on non-transferable tokens).
///
/// ```rust
/// use xrpl::types::transactions::nft::NFTokenCancelOffer;
/// let tx = NFTokenCancelOffer {
///     nftoken_offers: vec!["offer_id_1".to_string(), "offer_id_2".to_string()],
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenCancelOffer {
    /// List of ledger object IDs of NFT offers to cancel.
    #[serde(rename = "NFTokenOffers")]
    pub nftoken_offers: Vec<String>,
}

/// Creates a buy or sell offer for an NFToken.
///
/// Set the `tfSellNFToken` flag to create a sell offer; omit it for a buy offer.
/// For buy offers, `owner` must identify the current token holder.
///
/// ```rust
/// use xrpl::types::{Amount, transactions::nft::NFTokenCreateOffer};
/// let tx = NFTokenCreateOffer {
///     nftoken_id: "000B013A95F14B0044F78A264E41713C64B5F89242540EE208C3098E00000D65".to_string(),
///     amount: Amount::Xrpl("10000000".to_string()),
///     owner: None,
///     expiration: None,
///     destination: None,
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenCreateOffer {
    /// The 256-bit identifier of the NFToken.
    #[serde(rename = "NFTokenID")]
    pub nftoken_id: String,
    /// Offered price (XRP or issued currency).
    pub amount: Amount,
    /// Current token owner; required for buy offers where the submitter is not the owner.
    pub owner: Option<String>,
    /// Ripple-epoch time after which the offer expires.
    pub expiration: Option<u32>,
    /// Restricts acceptance to a specific account; omit to allow any account.
    pub destination: Option<String>,
}

/// Mints a new NFToken and places it in the submitter's NFToken page.
///
/// Set the `tfTransferable` flag to allow the token to be sold or transferred.
/// The royalty (`transfer_fee`) is enforced by the protocol on every secondary sale.
///
/// ```rust
/// use xrpl::types::transactions::nft::NFTokenMint;
/// let tx = NFTokenMint {
///     nftoken_taxon: 0,
///     issuer: None,
///     transfer_fee: Some(5000), // 5% royalty
///     uri: Some("68747470733a2f2f6578616d706c652e636f6d2f6e6674".to_string()),
/// };
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NFTokenMint {
    /// Arbitrary taxon (collection identifier) chosen by the issuer.
    #[serde(rename = "NFTokenTaxon")]
    pub nftoken_taxon: u32,
    /// Issuer account, if minting on behalf of another account (requires `NFTokenMinter` to be set).
    pub issuer: Option<String>,
    /// Royalty fee in units of 1/100,000 of a percent (0–50000).
    pub transfer_fee: Option<u16>,
    /// Hex-encoded URI pointing to the token's metadata (max 512 characters).
    #[serde(rename = "URI")]
    pub uri: Option<String>,
}
