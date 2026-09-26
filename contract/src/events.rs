use soroban_sdk::{Address, BytesN, Env, Symbol};

use crate::Subscription;

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscribedEventData {
    pub merchant: Address,
    pub amount: i128,
    pub interval: u64,
    pub ledger_sequence: u32,
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChargeEventData {
    pub merchant: Address,
    pub gross: i128,
    pub fee: i128,
    pub net: i128,
    pub charged_at: u64,
    pub ledger_sequence: u32,
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayPerUseEventData {
    pub merchant: Address,
    pub amount: i128,
    pub ledger_sequence: u32,
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelledEventData {
    pub ledger_sequence: u32,
}

#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancelledWithRefundEventData {
    pub refund_amount: i128,
    pub ledger_sequence: u32,
}

pub fn publish_subscribed(env: &Env, user: &Address, sub: &Subscription) {
    env.events().publish(
        (Symbol::new(env, "subscribed"), user.clone()),
        SubscribedEventData {
            merchant: sub.merchant.clone(),
            amount: sub.amount,
            interval: sub.interval,
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

pub fn publish_charged(
    env: &Env,
    user: &Address,
    sub: &Subscription,
    fee_amount: i128,
    charged_at: u64,
) {
    let net = sub.amount - fee_amount;
    env.events().publish(
        (Symbol::new(env, "charged"), user.clone()),
        ChargeEventData {
            merchant: sub.merchant.clone(),
            gross: sub.amount,
            fee: fee_amount,
            net,
            charged_at,
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

// ─────────────────────────────────────────────────────────────
// Admin transfer events
// ─────────────────────────────────────────────────────────────
//
// `transfer_admin` stages a pending admin takeover by storing `PendingAdmin`,
// but previously emitted nothing; only `accept_admin` fired `admin_transferred`.
// That left the first half of the two-step handoff invisible to indexers and
// monitoring. `admin_transfer_proposed` closes the gap by announcing the
// proposed address at staging time. The `accept_admin` event is unchanged.

/// Payload for `admin_transfer_proposed`. Carries the proposed new admin.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTransferProposedEventData {
    pub new_admin: Address,
    pub ledger_sequence: u32,
}

/// Publishes `admin_transfer_proposed(new_admin)` when a transfer is staged.
pub fn publish_admin_transfer_proposed(env: &Env, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, "admin_transfer_proposed"), new_admin.clone()),
        AdminTransferProposedEventData {
            new_admin: new_admin.clone(),
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

// ─────────────────────────────────────────────────────────────
// Subscription metadata events
// ─────────────────────────────────────────────────────────────
//
// `set_metadata` and `clear_metadata` mutate per-subscription labels without
// emitting events, while every other state-changing action publishes one.
// Indexers and the frontend's EventFeed cannot reconstruct label history
// without these events, leaving the event catalog incomplete.

/// Payload for `metadata_set`. Carries the subject (user) and the label that
/// was written.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataSetEventData {
    pub user: Address,
    pub label: BytesN<64>,
    pub ledger_sequence: u32,
}

/// Payload for `metadata_cleared`. Carries the subject (user) whose label was
/// removed.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataClearedEventData {
    pub user: Address,
    pub ledger_sequence: u32,
}

/// Publishes `metadata_set(user)` with the written label.
pub fn publish_metadata_set(env: &Env, user: &Address, label: &BytesN<64>) {
    env.events().publish(
        (Symbol::new(env, "metadata_set"), user.clone()),
        MetadataSetEventData {
            user: user.clone(),
            label: label.clone(),
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

/// Publishes `metadata_cleared(user)`.
pub fn publish_metadata_cleared(env: &Env, user: &Address) {
    env.events().publish(
        (Symbol::new(env, "metadata_cleared"), user.clone()),
        MetadataClearedEventData {
            user: user.clone(),
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

// ─────────────────────────────────────────────────────────────
// Merchant revenue audit events
// ─────────────────────────────────────────────────────────────
//
// `reset_merchant_revenue` and `prune_merchant_revenue_days` destroy merchant
// revenue history. Without an on-chain event, ops tooling and indexers cannot
// distinguish an operator reset from a bug or archival, and annual/journal
// reconciliation has no trail. These events close that gap.

/// Payload for `merchant_revenue_reset`. Emitted when an operator resets a
/// merchant's revenue history.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantRevenueResetEventData {
    pub merchant: Address,
    pub ledger_sequence: u32,
}

/// Payload for `merchant_revenue_pruned`. `removed_days` is the number of day
/// buckets removed, so reconciliation can account for the pruned history.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerchantRevenuePrunedEventData {
    pub merchant: Address,
    pub removed_days: u32,
    pub ledger_sequence: u32,
}

/// Publishes `merchant_revenue_reset(merchant)`.
pub fn publish_merchant_revenue_reset(env: &Env, merchant: &Address) {
    env.events().publish(
        (Symbol::new(env, "merchant_revenue_reset"), merchant.clone()),
        MerchantRevenueResetEventData {
            merchant: merchant.clone(),
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

/// Publishes `merchant_revenue_pruned(merchant)` with the number of removed
/// day buckets.
pub fn publish_merchant_revenue_pruned(env: &Env, merchant: &Address, removed_days: u32) {
    env.events().publish(
        (Symbol::new(env, "merchant_revenue_pruned"), merchant.clone()),
        MerchantRevenuePrunedEventData {
            merchant: merchant.clone(),
            removed_days,
            ledger_sequence: env.ledger().sequence(),
        },
    );
}

// ─────────────────────────────────────────────────────────────
// Batch charge skip summary
// ─────────────────────────────────────────────────────────────
//
// `batch_charge` reports per-user outcomes in its return value, which only the
// caller of the transaction sees. Event-driven consumers (scripts/indexer.ts,
// scripts/watch-events.ts) therefore had no on-chain signal for a batch where
// subscriptions were paused, cancelled, missing, or past their grace window.
//
// This event closes that gap with ONE summary per batch instead of one event
// per skipped user: per-user emission would scale event fees and ledger
// footprint with batch size, and the not-due case (`Skipped`) is the common,
// uninteresting outcome that would dominate the stream. Per-user attribution
// stays available off-chain via the return value and `get_batch_charge_estimate`.
//
// Emission is conditional: the event fires only when at least one *interesting*
// outcome occurred (no_subscription / inactive / paused / grace_elapsed /
// allowance_insufficient). An all-charged or all-not-due batch emits nothing,
// so the steady state costs exactly what it did before.

/// Aggregate outcome counts for a single `batch_charge` call.
///
/// `charged + not_due + no_subscription + inactive + grace_elapsed + paused
/// + allowance_insufficient == total`.
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchChargeSkipsEventData {
    /// Addresses submitted in the batch.
    pub total: u32,
    /// `ChargeResult::Charged`
    pub charged: u32,
    /// `ChargeResult::Skipped` — interval has not elapsed yet.
    pub not_due: u32,
    /// `ChargeResult::NoSubscription`
    pub no_subscription: u32,
    /// `ChargeResult::Inactive`
    pub inactive: u32,
    /// `ChargeResult::Paused`
    pub paused: u32,
    /// `ChargeResult::GracePeriodElapsed`
    pub grace_elapsed: u32,
    /// `ChargeResult::AllowanceInsufficient` — the subscriber's allowance is
    /// below the gross amount. This is the alerting case: the subscription is
    /// still active and will keep failing until the subscriber re-approves.
    pub allowance_insufficient: u32,
    pub ledger_sequence: u32,
}

/// Publishes the `batch_charge_skips` summary. Callers must only invoke this
/// when at least one interesting (non-`Charged`, non-`Skipped`) outcome occurred.
pub fn publish_batch_charge_skips(env: &Env, data: BatchChargeSkipsEventData) {
    env.events().publish(
        (Symbol::new(env, "batch_charge_skips"),),
        data,
    );
}
