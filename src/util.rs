//! Account utility helpers - thin wrappers around common `account_info` and `server_state` queries.

use crate::{
    request::{
        server_state::ServerStateRequest,
        account_info::{AccountInfoRequest, AccountInfoResponse},
    },
    types::AccountFlags,
    Client, XrplError,
};

/// All three XRP balances for an account in a single pair of concurrent requests.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::Balances};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let b = xrpl::util::account_balances(&client, "rAccount...").await?;
/// println!("total: {} | reserved: {} | available: {}", b.total, b.reserved, b.available);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Balances {
    /// Total XRP balance in drops.
    pub total: u64,
    /// XRP locked in reserves (base reserve + per-object owner reserve increment) in drops.
    pub reserved: u64,
    /// Spendable XRP in drops (`total` minus `reserved`).
    pub available: u64,
}

/// Fetches `account_info` from the validated ledger; maps `actNotFound` to `None`.
async fn fetch_account_info(
    client: &Client,
    account: &str,
) -> Result<Option<AccountInfoResponse>, XrplError> {
    let req = AccountInfoRequest {
        account: account.to_string(),
        ledger_index: Some(serde_json::json!("validated")),
        ..Default::default()
    };
    match client.request(&req).await {
        Ok(resp) => Ok(Some(resp.result()?)),
        Err(XrplError::ApiError { ref error, .. })
            if error == "actNotFound" =>
        {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

fn not_found(account: &str) -> XrplError {
    XrplError::ApiError {
        error: "actNotFound".to_string(),
        error_code: Some(19),
        error_message: Some(format!("Account {account} not found")),
    }
}

/// Next valid sequence number for single transactions or when you wait for validation between submissions.
///
/// Reads from the current open ledger so the value is immediately usable without waiting
/// for the previous transaction to be validated.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::next_sequence};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let seq = next_sequence(&client, "rAccount...").await?;
/// # Ok(())
/// # }
/// ```
pub async fn next_sequence(
    client: &Client,
    account: &str,
) -> Result<u32, XrplError> {
    let req = AccountInfoRequest {
        account: account.to_string(),
        ledger_index: Some(serde_json::json!("current")),
        ..Default::default()
    };
    Ok(client.request(&req).await?.result()?.account_data.sequence)
}

/// Next sequence number for rapid multi-transaction workflows.
///
/// Uses the tx queue (`queue: true`) to find the highest queued sequence and returns
/// `highest + 1`. Falls back to `account_data.sequence` when nothing is queued.
/// This lets you submit a burst of transactions without waiting for each to be validated.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::next_queued_sequence};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let seq = next_queued_sequence(&client, "rAccount...").await?;
/// # Ok(())
/// # }
/// ```
pub async fn next_queued_sequence(
    client: &Client,
    account: &str,
) -> Result<u32, XrplError> {
    let req = AccountInfoRequest {
        account: account.to_string(),
        ledger_index: Some(serde_json::json!("current")),
        queue: Some(true),
        ..Default::default()
    };
    let resp = client.request(&req).await?.result()?;
    let next = resp
        .queue_data
        .and_then(|q| q.highest_sequence)
        .map(|h| h + 1)
        .unwrap_or(resp.account_data.sequence);
    Ok(next)
}

/// All three XRP balances for an account in a single pair of concurrent requests.
///
/// Issues `account_info` and `server_state` concurrently, then computes all three
/// values from the combined result. Use this when more than one balance is needed
/// to avoid redundant network round-trips.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::account_balances};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let b = account_balances(&client, "rAccount...").await?;
/// println!("total: {} | reserved: {} | available: {}", b.total, b.reserved, b.available);
/// # Ok(())
/// # }
/// ```
pub async fn account_balances(
    client: &Client,
    account: &str,
) -> Result<Balances, XrplError> {
    let server_state_req = ServerStateRequest::default();
    let (info_result, state_result) = tokio::join!(
        fetch_account_info(client, account),
        client.request(&server_state_req),
    );
    let info = info_result?.ok_or_else(|| not_found(account))?;
    let ledger =
        state_result?.result()?.state.validated_ledger.ok_or_else(|| {
            XrplError::ParseError(
                "ServerState did not return a validated_ledger".to_string(),
            )
        })?;

    let total = info
        .account_data
        .balance
        .parse::<u64>()
        .map_err(|e| XrplError::ParseError(e.to_string()))?;
    let reserved = ledger.reserve_base
        + ledger.reserve_inc * u64::from(info.account_data.owner_count);

    Ok(Balances { total, reserved, available: total.saturating_sub(reserved) })
}

/// Total XRP balance in drops.
///
/// When only the raw balance is needed, this avoids the extra `server_state` request
/// required by [`account_balances`]. Use [`account_balances`] when reserve or
/// spendable amounts are also needed.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::xrp_balance};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let drops = xrp_balance(&client, "rAccount...").await?;
/// println!("{} drops ({} XRP)", drops, drops / 1_000_000);
/// # Ok(())
/// # }
/// ```
pub async fn xrp_balance(
    client: &Client,
    account: &str,
) -> Result<u64, XrplError> {
    fetch_account_info(client, account)
        .await?
        .ok_or_else(|| not_found(account))?
        .account_data
        .balance
        .parse::<u64>()
        .map_err(|e| XrplError::ParseError(e.to_string()))
}

/// Spendable XRP balance in drops - total balance minus reserves.
///
/// Delegates to [`account_balances`]. Use that directly when reserved or total
/// amounts are also needed.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::available_balance};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let spendable = available_balance(&client, "rAccount...").await?;
/// # Ok(())
/// # }
/// ```
pub async fn available_balance(
    client: &Client,
    account: &str,
) -> Result<u64, XrplError> {
    Ok(account_balances(client, account).await?.available)
}

/// XRP locked in reserves in drops - base reserve plus per-object owner reserve increment.
///
/// Delegates to [`account_balances`]. Use that directly when total or spendable
/// amounts are also needed.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::reserved_balance};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let locked = reserved_balance(&client, "rAccount...").await?;
/// println!("{} drops locked in reserves", locked);
/// # Ok(())
/// # }
/// ```
pub async fn reserved_balance(
    client: &Client,
    account: &str,
) -> Result<u64, XrplError> {
    Ok(account_balances(client, account).await?.reserved)
}

/// Whether `account` has been funded on the validated ledger.
///
/// Sending to an unfunded account requires a payment that meets the base reserve;
/// otherwise the transaction fails with `tecNO_DST`.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::account_exists};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// if !account_exists(&client, "rNewAccount...").await? {
///     println!("Account not yet funded - payment must meet base reserve");
/// }
/// # Ok(())
/// # }
/// ```
pub async fn account_exists(
    client: &Client,
    account: &str,
) -> Result<bool, XrplError> {
    Ok(fetch_account_info(client, account).await?.is_some())
}

/// Number of objects the account owns on the validated ledger.
///
/// Each owned object increases the reserve requirement by the owner reserve increment.
/// Useful for custom reserve calculations with known or cached reserve constants.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::owner_count};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let count = owner_count(&client, "rAccount...").await?;
/// # Ok(())
/// # }
/// ```
pub async fn owner_count(
    client: &Client,
    account: &str,
) -> Result<u32, XrplError> {
    Ok(fetch_account_info(client, account)
        .await?
        .ok_or_else(|| not_found(account))?
        .account_data
        .owner_count)
}

/// Active account flags from the validated ledger.
///
/// # Example
/// ```no_run
/// use xrpl::{Client, util::account_flags};
/// use xrpl::types::AccountFlag;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let client = Client::new("wss://xrplcluster.com");
/// let flags = account_flags(&client, "rAccount...").await?;
/// if flags.has(AccountFlag::RequireDest) {
///     println!("destination tag required");
/// }
/// # Ok(())
/// # }
/// ```
pub async fn account_flags(
    client: &Client,
    account: &str,
) -> Result<AccountFlags, XrplError> {
    Ok(fetch_account_info(client, account)
        .await?
        .ok_or_else(|| not_found(account))?
        .account_data
        .flags)
}
