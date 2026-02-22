/// Monotonic nonce tracker used for authenticated requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceTracker {
    current: u64,
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
    pub fn next_nonce(&mut self) -> u64 {
        self.current = self.current.saturating_add(1);
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::NonceTracker;

    #[test]
    fn unit_nonce_tracker_advances_monotonically() {
        let mut tracker = NonceTracker::new(0);
        assert_eq!(tracker.next_nonce(), 1);
        assert_eq!(tracker.next_nonce(), 2);
        assert_eq!(tracker.current(), 2);
    }
}
