# Protocol and UI Limits

Authoritative **admin batch-operation** caps as implemented today. Values are taken from the contract constants and the live frontend panels — not from older issue text.

There is **no** `frontend/src/shared/constants.ts`. Amount/interval UI constants live in [`frontend/src/constants.ts`](../frontend/src/constants.ts) (`CONTRACT_LIMITS`). Batch size caps used by admin panels are local constants in those components, kept in lockstep with the contract symbols named below.

**Daily spending caps** (pay-per-use) are a different topic: [`DAILY-LIMITS.md`](DAILY-LIMITS.md).

---

## Subscription interval bounds

| Constant                  | Value                  | Source                          | Notes |
| ------------------------- | ---------------------- | ------------------------------- | ----- |
| `MAX_SUBSCRIPTION_INTERVAL` | **12 623 040 000 s** (≈ 400 years) | `contract/src/lib.rs` | Upper bound enforced in `validate_interval`. Interval above this returns `IntervalExceedsMaximum` (error 46). |
| Min interval (default)    | **60 s** (1 minute)    | `contract/src/min_interval.rs`  | Admin-configurable via `set_min_interval`. |

**Why the cap exists:** Soroban timestamps are `u64` Unix seconds. `last_charged + interval` and `last_charged + interval + grace_period` are both u64 additions. A near-`u64::MAX` interval overflows those expressions, causing every `charge()` call for that subscription to abort — inside `batch_charge` that historically aborted the whole batch (a self-inflicted DoS / keeper hazard). All timestamp arithmetic now uses `saturating_add` so extreme legacy values (injected directly into storage) cannot abort a batch; they are simply treated as "not yet due" forever.

**Legacy subscriptions:** Any subscription created before this cap was introduced with an interval above `MAX_SUBSCRIPTION_INTERVAL` can still be cancelled via `cancel(user)` but cannot be charged. The subscriber must re-subscribe with a valid interval to restore chargeability.

---

## Admin batch operations (UI ↔ contract)

| Operation | Contract method | UI cap (addresses / call) | Contract cap (addresses / call) | UI source | Contract source | Alignment |
| --------- | --------------- | -------------------------:| -------------------------------:| --------- | --------------- | --------- |
| Charge | `batch_charge` | **None** — merchant dashboard sends every due subscriber in one tx | Default **50** (`MAX_BATCH_SIZE`); instance override `get_max_batch_size` / `set_max_batch_size`, hard ceiling **200** | [`MerchantDashboard.tsx`](../frontend/src/components/MerchantDashboard.tsx) `handleBatchCharge` (no chunking). No admin charge panel. | [`batch.rs`](../contract/src/batch.rs) `MAX_BATCH_SIZE`; [`lib.rs`](../contract/src/lib.rs) `MAX_BATCH_SIZE_CEILING`, `DataKey::MaxBatchSize` | **Not aligned.** UI does not clamp to 50. Exceeding the effective cap panics `BatchTooLarge` (20). |
| Cancel | `batch_cancel` | **No UI** | **25** (`MAX_BATCH_PAUSE_SUBSCRIPTIONS`) | No `BatchCancel` panel under `frontend/src/components/admin/` | [`batch.rs`](../contract/src/batch.rs) `batch_cancel`; [`lib.rs`](../contract/src/lib.rs) `MAX_BATCH_PAUSE_SUBSCRIPTIONS` | N/A in UI. Same numeric cap as pause. |
| Pause | `batch_pause_subscriptions` | **25** (chunks larger lists) | **25** (literal `max_batch: u32 = 25`, same as `MAX_BATCH_PAUSE_SUBSCRIPTIONS`) | [`BatchPausePanel.tsx`](../frontend/src/components/admin/BatchPausePanel.tsx) `MAX_PAUSE_BATCH = 25` | [`lib.rs`](../contract/src/lib.rs) `batch_pause_subscriptions` | **Aligned** |
| Whitelist add | `whitelist_batch_add` | **50** (chunks larger lists) | Default **50** (`MAX_WHITELIST_BATCH_SIZE`); override `get_max_whitelist_batch_size` / `set_max_whitelist_batch_size`; ceiling **200** | [`BatchWhitelistPanel.tsx`](../frontend/src/components/admin/BatchWhitelistPanel.tsx) `MAX_WHITELIST_BATCH = 50` | [`lib.rs`](../contract/src/lib.rs) `MAX_WHITELIST_BATCH_SIZE`; [`whitelist.rs`](../contract/src/whitelist.rs) | **Aligned to the default.** If an admin raises the on-chain cap above 50, the UI still chunks at 50 (safe, smaller than contract). If the on-chain cap is lowered below 50, the UI can submit over-size chunks until it is updated. |
| Whitelist remove | `whitelist_batch_remove` | **50** (same panel / constant) | Same whitelist knob as add | Same as add | Same as add | Same as add |

Admin UI host: [`pages/AdminDashboard.tsx`](../frontend/src/pages/AdminDashboard.tsx) mounts pause + whitelist (and repair / protocol stats). Component index: [`FRONTEND-COMPONENTS.md`](FRONTEND-COMPONENTS.md).

### Related batch entrypoints (no admin panel)

| Operation | Contract method | Contract cap | Notes |
| --------- | --------------- | ------------ | ----- |
| TTL extend | `batch_extend_subscription_ttl` | Same as charge (`get_max_batch_size`, default 50, ceiling 200) | No frontend caller |
| Charge estimate | `get_batch_charge_estimate` | Hard **200** in `lib.rs` (not `get_max_batch_size`) | Used by keeper dry-run / health-check, not an admin panel |
| Merchant statuses | `get_merchant_statuses` | Whitelist batch cap | Shares `require_batch_within_limit`; no frontend caller |

`MAX_BATCH_SIZE` (charge/extend) and `MAX_WHITELIST_BATCH_SIZE` are **separate** knobs. Both are bounded by `MAX_BATCH_SIZE_CEILING` (200). Pause and cancel are **not** configurable; they are fixed at 25.

---

## How to re-verify this table

```bash
# Contract defaults
rg -n "MAX_BATCH_SIZE|MAX_BATCH_PAUSE_SUBSCRIPTIONS|MAX_WHITELIST_BATCH_SIZE|MAX_BATCH_SIZE_CEILING" contract/src/lib.rs contract/src/batch.rs contract/src/whitelist.rs

# UI caps
rg -n "MAX_PAUSE_BATCH|MAX_WHITELIST_BATCH" frontend/src/components/admin/
```

`frontend/src/constants.ts` does **not** currently export batch sizes.

---

## Related

- [`DAILY-LIMITS.md`](DAILY-LIMITS.md) — `pay_per_use` daily cap
- [`API.md`](API.md) — `batch_charge`, `batch_cancel`, `batch_pause_subscriptions`, whitelist batch methods
- [`ERROR-CODES.md`](ERROR-CODES.md) — `BatchTooLarge` (20), `InvalidBatchSize` (29)
