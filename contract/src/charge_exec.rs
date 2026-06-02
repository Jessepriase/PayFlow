use soroban_sdk::{Address, Env};

use crate::batch::ChargeResult;
use crate::events;
use crate::fee;
use crate::merchant_stats;
use crate::subscription_history;
use crate::{extend_subscription_ttl, DataKey, Subscription};

/// Batch pre-check: returns `Ok(())` when a charge may proceed, or the skip result.
pub fn precheck_charge(
    sub: &Subscription,
    now: u64,
    grace_period: u64,
) -> Result<(), ChargeResult> {
    if !sub.active {
        return Err(ChargeResult::Inactive);
    }
    if sub.paused {
        return Err(ChargeResult::Paused);
    }
    if now < sub.last_charged + sub.interval {
        return Err(ChargeResult::Skipped);
    }
    if grace_period > 0 && now > sub.last_charged + sub.interval + grace_period {
        return Err(ChargeResult::GracePeriodElapsed);
    }
    Ok(())
}

/// Fee-aware transfer, bookkeeping, and persistence shared by `charge()` and `batch_charge()`.
/// Returns the protocol fee deducted from the subscription amount.
pub fn execute_charge(
    env: &Env,
    user: &Address,
    key: &DataKey,
    sub: &mut Subscription,
    now: u64,
) -> i128 {
    let fee_amount = fee::transfer_subscription_charge(env, user, sub);
    let net = sub.amount - fee_amount;

    merchant_stats::increment_revenue_with_daily(env, &sub.merchant, net);

    sub.last_charged = now;
    env.storage().persistent().set(key, sub);
    extend_subscription_ttl(env, user);
    subscription_history::record_charge(env, user, now);
    events::publish_charged(env, user, sub, fee_amount, now);

    fee_amount
}
