/// Monotonic nonce tracker used for authenticated requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceTracker {
    current: u64,
}

/// Error returned when the nonce tracker cannot advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonceTrackerError {
    /// Tracker is exhausted and cannot produce another nonce.
    Exhausted,
}

impl NonceTracker {
    /// Builds a tracker with an initial nonce value.
    pub fn new(initial: u64) -> Self {
        Self { current: initial }
    }

    /// Returns the current nonce value.
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Advances and returns the next nonce value.
    pub fn next_nonce(&mut self) -> Result<u64, NonceTrackerError> {
        let next = self
            .current
            .checked_add(1)
            .ok_or(NonceTrackerError::Exhausted)?;
        self.current = next;
        Ok(self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::{NonceTracker, NonceTrackerError};

    #[test]
    fn unit_nonce_tracker_advances_monotonically() {
        let mut tracker = NonceTracker::new(0);
        assert_eq!(tracker.next_nonce(), Ok(1));
        assert_eq!(tracker.next_nonce(), Ok(2));
        assert_eq!(tracker.current(), 2);
    }

    #[test]
    fn regression_nonce_tracker_overflow_must_not_reuse_nonce_value() {
        // Regression: #5907
        let mut tracker = NonceTracker::new(u64::MAX - 1);
        assert_eq!(tracker.next_nonce(), Ok(u64::MAX));
        assert_ne!(
            tracker.next_nonce(),
            Ok(u64::MAX),
            "overflow path must not silently emit a duplicate nonce"
        );
        assert_eq!(tracker.next_nonce(), Err(NonceTrackerError::Exhausted));
    }
}
