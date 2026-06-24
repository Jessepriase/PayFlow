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
    fn test_require_positive_amount_positive() {
        require_positive_amount(1);
        require_positive_amount(100);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_require_positive_amount_negative() {
        require_positive_amount(0);
    }

    #[test]
    #[should_panic(expected = "amount must be positive")]
    fn test_require_positive_amount_negative_signed() {
        require_positive_amount(-5);
    }

    #[test]
    fn test_require_positive_interval_positive() {
        require_positive_interval(1);
        require_positive_interval(60);
    }

    #[test]
    #[should_panic(expected = "interval must be positive")]
    fn test_require_positive_interval_negative() {
        require_positive_interval(0);
    }

    #[test]
    fn test_require_active_subscription_positive() {
        require_active_subscription(true);
    }

    #[test]
    #[should_panic(expected = "subscription is not active")]
    fn test_require_active_subscription_negative() {
        require_active_subscription(false);
    }

    #[test]
    fn test_require_charge_interval_elapsed_positive() {
        require_charge_interval_elapsed(100, 40, 60);
        require_charge_interval_elapsed(150, 40, 60);
    }

    #[test]
    #[should_panic(expected = "interval not elapsed yet")]
    fn test_require_charge_interval_elapsed_negative() {
        require_charge_interval_elapsed(99, 40, 60);
    }
}
