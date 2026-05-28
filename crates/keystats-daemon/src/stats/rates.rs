use std::collections::VecDeque;
use std::time::Instant;

/// 1-second sliding window rate tracker.
#[allow(dead_code)]
pub struct RateTracker {
    window: VecDeque<Instant>,
}

#[allow(dead_code)]
impl RateTracker {
    pub fn new() -> Self {
        Self {
            window: VecDeque::new(),
        }
    }

    /// Record a single event at the current time.
    pub fn record(&mut self) {
        let now = Instant::now();
        self.window.push_back(now);
        self.prune(now);
    }

    /// Return the number of events in the last second.
    pub fn current_rate(&mut self) -> u32 {
        let now = Instant::now();
        self.prune(now);
        self.window.len() as u32
    }

    fn prune(&mut self, now: Instant) {
        while self
            .window
            .front()
            .is_some_and(|t| now.duration_since(*t).as_secs_f64() > 1.0)
        {
            self.window.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn rate_tracker_starts_zero() {
        let mut rt = RateTracker::new();
        assert_eq!(rt.current_rate(), 0);
    }

    #[test]
    fn rate_tracker_counts_events() {
        let mut rt = RateTracker::new();
        for _ in 0..10 {
            rt.record();
        }
        assert_eq!(rt.current_rate(), 10);
    }

    #[test]
    fn rate_events_expire_after_one_second() {
        let mut rt = RateTracker::new();
        for _ in 0..5 {
            rt.record();
        }
        sleep(Duration::from_secs(1) + Duration::from_millis(100));
        assert_eq!(rt.current_rate(), 0);
    }
}
