//! Time-windowed metric aggregation for streaming telemetry.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Sliding-window accumulator keyed by wall-clock time.
pub struct RollingWindow {
    window: Duration,
    buckets: VecDeque<(Instant, f64)>,
}

/// Summary statistics computed over a rolling window.
#[derive(Debug, Clone)]
pub struct WindowStats {
    pub sum: f64,
    pub mean: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub rate: f64,
}

impl RollingWindow {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            buckets: VecDeque::new(),
        }
    }

    /// Push a new observation and evict entries outside the window.
    pub fn push(&mut self, value: f64) {
        let now = Instant::now();
        self.buckets.push_back((now, value));
        self.evict(now);
    }

    /// Evict all entries older than the window relative to `now`.
    fn evict(&mut self, now: Instant) {
        while let Some(&(ts, _)) = self.buckets.front() {
            if now.duration_since(ts) > self.window {
                self.buckets.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn sum(&self) -> f64 {
        self.buckets.iter().map(|(_, v)| v).sum()
    }

    pub fn mean(&self) -> f64 {
        if self.buckets.is_empty() {
            return 0.0;
        }
        self.sum() / self.buckets.len() as f64
    }

    /// Linear-interpolation percentile (p in [0, 100]).
    pub fn percentile(&self, p: f64) -> f64 {
        if self.buckets.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f64> = self.buckets.iter().map(|(_, v)| *v).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        if n == 1 {
            return sorted[0];
        }
        let rank = (p / 100.0) * (n - 1) as f64;
        let lo = rank.floor() as usize;
        let hi = rank.ceil() as usize;
        let frac = rank - lo as f64;
        sorted[lo] + frac * (sorted[hi] - sorted[lo])
    }

    /// Observed events per second over the window.
    pub fn rate(&self) -> f64 {
        let secs = self.window.as_secs_f64();
        if secs == 0.0 {
            return 0.0;
        }
        self.buckets.len() as f64 / secs
    }

    /// Compute all standard stats in one pass.
    pub fn stats(&self) -> WindowStats {
        WindowStats {
            sum: self.sum(),
            mean: self.mean(),
            p50: self.percentile(50.0),
            p95: self.percentile(95.0),
            p99: self.percentile(99.0),
            rate: self.rate(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rolling_window_sum() {
        let mut w = RollingWindow::new(Duration::from_secs(60));
        w.push(1.0);
        w.push(2.0);
        w.push(3.0);
        assert!((w.sum() - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_rolling_window_mean() {
        let mut w = RollingWindow::new(Duration::from_secs(60));
        w.push(10.0);
        w.push(20.0);
        assert!((w.mean() - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_rolling_window_percentile() {
        let mut w = RollingWindow::new(Duration::from_secs(60));
        for i in 1..=10 {
            w.push(i as f64);
        }
        // p99 should be near 10.0
        let p99 = w.percentile(99.0);
        assert!(p99 >= 9.0 && p99 <= 10.0, "p99={p99}");
        // p50 should be near 5.5 (linear interp between 5 and 6)
        let p50 = w.percentile(50.0);
        assert!((p50 - 5.5).abs() < 0.1, "p50={p50}");
    }

    #[test]
    fn test_rate_calculation() {
        let mut w = RollingWindow::new(Duration::from_secs(1));
        for _ in 0..60 {
            w.push(1.0);
        }
        // 60 events over a 1-second window → rate = 60.0
        assert!((w.rate() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_empty_window() {
        let w = RollingWindow::new(Duration::from_secs(60));
        assert_eq!(w.sum(), 0.0);
        assert_eq!(w.mean(), 0.0);
        assert_eq!(w.percentile(99.0), 0.0);
        assert_eq!(w.rate(), 0.0);
    }

    #[test]
    fn test_eviction() {
        let mut w = RollingWindow::new(Duration::from_millis(50));
        w.push(100.0);
        std::thread::sleep(Duration::from_millis(100));
        // Trigger eviction by pushing a new value
        w.push(1.0);
        // Old entry should have been evicted
        assert!((w.sum() - 1.0).abs() < 1e-9);
    }
}
