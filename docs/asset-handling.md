# XRPL Asset Handling Instructions

## Asset Classification Logic
Agents must distinguish between three distinct asset types based on their data structure to ensure correct transaction construction.

### Type A: Native Asset (XRP)
*   **Identification:** Represented as a single string or integer (in "drops").
*   **Conversion:** $1 \text{ XRP} = 1,000,000 \text{ drops}$.
*   **Logic:** Native to the ledger. No issuer or Trust Lines required.

### Type B: Issued Currencies (IOU)
*   **Identification:** A JSON "Triple" consisting of `currency` (3-char code/40-char hex), `issuer` (r-address), and `value`.
*   **Mechanism:** Operates via **Trust Lines**. Represents a debt relationship with the counterparty (Issuer).
*   **Behavior:** Subject to "Rippling" unless the `NoRipple` flag is explicitly enabled.

### Type C: Multi-Purpose Tokens (MPT)
*   **Identification:** A JSON pair consisting of `mpt_issuance_id` (48-char hex) and `value`.
*   **Mechanism:** Self-contained ledger objects. **No Trust Lines required.**
*   **Precision:** Purely integer-based math ($0$ to $2^{63}-1$).

---

## Data Validation Rules for SDKs
Enforce the following constraints during object instantiation:

| Field | Validation Requirement |
| :--- | :--- |
| **MPT Value** | Must be a positive integer string. No decimals or scientific notation allowed. |
| **MPT ID** | Must be exactly 48 characters, hexadecimal format. |
| **IOU Value** | Supports scientific notation (e.g., `1.2e-5`) and floating-point strings. |
| **IOU Code** | 3-character standard or 40-character non-standard hex. |

---

## AMM & DEX Interaction
*   **IOU Pairs:** Linked via `CurrencyCode:Issuer`. Bids must match the specific issuer address to target the correct liquidity pool.
*   **MPT Pairs:** Linked via the unique `mpt_issuance_id`. Scale and metadata are resolved from the issuance object.
*   **Precision Management:** Always calculate price impact using the `AssetScale` defined in the MPT issuance metadata to avoid order-of-magnitude errors.

---

## Metadata Resolution Strategy
Since some explorers do not auto-decode metadata:
1.  Query the `MPTokenIssuance` object.
2.  Extract the `MPTokenMetadata` hex string.
3.  Convert Hex to UTF-8/JSON.
4.  **Note:** Metadata is **immutable** once created. Verify all fields (especially `uri`) before final submission.
