pub fn require_positive_amount(amount: i128) {
    assert!(amount > 0, "amount must be positive");
}

pub fn require_positive_interval(interval: u64) {
    assert!(interval > 0, "interval must be positive");
}

pub fn require_active_subscription(active: bool) {
    assert!(active, "subscription is not active");
}

pub fn require_charge_interval_elapsed(now: u64, last_charged: u64, interval: u64) {
    assert!(now >= last_charged + interval, "interval not elapsed yet");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_positive_amount_accepts_positive() {
        require_positive_amount(1);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_require_positive_amount_panics_on_zero() {
        require_positive_amount(0);
    }

    #[test]
    fn test_require_positive_interval_accepts_positive() {
        require_positive_interval(60);
    }

    #[test]
    #[should_panic(expected = "interval must be positive")]
    fn test_require_positive_interval_panics_on_zero() {
        require_positive_interval(0);
    }

    #[test]
    fn test_require_active_subscription_accepts_true() {
        require_active_subscription(true);
    }

    #[test]
    #[should_panic(expected = "subscription is not active")]
    fn test_require_active_subscription_panics_on_false() {
        require_active_subscription(false);
    }

    #[test]
    fn test_require_charge_interval_elapsed_accepts_elapsed_interval() {
        require_charge_interval_elapsed(100, 40, 60);
    }

    #[test]
    #[should_panic(expected = "interval not elapsed yet")]
    fn test_require_charge_interval_elapsed_panics_if_too_early() {
        require_charge_interval_elapsed(99, 40, 60);
    }
}
