// Copyright © 2020-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use instant::{Duration, Instant};

/// Useful timer to get delta time, and previous time
pub struct Timer {
    prev: Instant,
    curr: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let prev = Instant::now();
        let curr = Instant::now();
        Self { prev, curr }
    }

    /// Returns delta time
    pub fn get_delta(&mut self) -> Duration {
        self.curr = Instant::now();
        let delta = self.curr - self.prev;
        self.prev = self.curr;
        delta
    }

    /// Returns the time of last `get_delta()`
    pub fn get_prev(&self) -> Instant {
        self.prev
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timer_delta_and_prev() {
        let mut timer = Timer::new();
        let prev = timer.get_prev();
        sleep(Duration::from_millis(10));
        let delta = timer.get_delta();
        assert!(delta >= Duration::from_millis(10));
        let prev2 = timer.get_prev();
        assert!(prev2 > prev);
    }

    #[test]
    fn test_timer_default() {
        let mut timer = Timer::default();
        sleep(Duration::from_millis(5));
        let delta = timer.get_delta();
        assert!(delta >= Duration::from_millis(5));
    }

    #[test]
    fn test_multiple_deltas() {
        let mut timer = Timer::new();
        sleep(Duration::from_millis(3));
        let d1 = timer.get_delta();
        sleep(Duration::from_millis(4));
        let d2 = timer.get_delta();
        assert!(d2 >= Duration::from_millis(4));
        assert!(d1 < d2 || d1 > Duration::from_millis(0));
    }
}
