use chrono::Utc;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Message timestamp too far in future")]
    FutureTimestamp,
    #[error("Message timestamp expired")]
    ExpiredTimestamp,
    #[error("Message replay detected")]
    ReplayDetected,
}

pub type Result<T> = std::result::Result<T, ReplayError>;

/// Detects replay attacks using timestamp validation and nonce caching
pub struct ReplayDetector {
    cache: Arc<Mutex<LruCache<(String, u64), ()>>>,
    max_age_seconds: u64,
    clock_skew_tolerance: u64,
}

impl std::fmt::Debug for ReplayDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReplayDetector")
            .field("max_age_seconds", &self.max_age_seconds)
            .field("clock_skew_tolerance", &self.clock_skew_tolerance)
            .field("cache_size", &self.cache.lock().unwrap().len())
            .finish()
    }
}

impl ReplayDetector {
    pub fn new(cache_size: usize, max_age_seconds: u64) -> Self {
        let cache_size = NonZeroUsize::new(cache_size).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            max_age_seconds,
            clock_skew_tolerance: 120, // 2 minutes default
        }
    }

    pub fn check_message(&self, message_id: &str, timestamp: u64, nonce: u128) -> Result<()> {
        let now = Utc::now().timestamp() as u64;

        // Check timestamp freshness
        if timestamp > now + self.clock_skew_tolerance {
            return Err(ReplayError::FutureTimestamp);
        }
        if now > timestamp && (now - timestamp) > self.max_age_seconds {
            return Err(ReplayError::ExpiredTimestamp);
        }

        // Check for replay
        // Key is combination of message_id and nonce to ensure uniqueness
        // We also include timestamp to differentiate same message sent at different times (if allowed)
        // But typically message_id should be unique.
        // Using (message_id, nonce) as key.
        // Actually, nonce should be enough if it's globally unique, but per-message nonce is safer.
        let key = (format!("{}:{}", message_id, nonce), timestamp);

        let mut cache = self.cache.lock().unwrap();
        if cache.contains(&key) {
            return Err(ReplayError::ReplayDetected);
        }

        cache.put(key, ());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_detector_creation() {
        let _detector = ReplayDetector::new(100, 300);
    }

    #[test]
    fn test_valid_message() {
        let detector = ReplayDetector::new(100, 300);
        let now = Utc::now().timestamp() as u64;
        assert!(detector.check_message("msg1", now, 123).is_ok());
    }

    #[test]
    fn test_future_timestamp() {
        let detector = ReplayDetector::new(100, 300);
        let future = Utc::now().timestamp() as u64 + 300; // 5 mins in future
        assert!(matches!(
            detector.check_message("msg1", future, 123),
            Err(ReplayError::FutureTimestamp)
        ));
    }

    #[test]
    fn test_expired_timestamp() {
        let detector = ReplayDetector::new(100, 300);
        let past = Utc::now().timestamp() as u64 - 301; // 5 mins + 1 sec ago
        assert!(matches!(
            detector.check_message("msg1", past, 123),
            Err(ReplayError::ExpiredTimestamp)
        ));
    }

    #[test]
    fn test_replay_detection() {
        let detector = ReplayDetector::new(100, 300);
        let now = Utc::now().timestamp() as u64;

        // First time ok
        assert!(detector.check_message("msg1", now, 123).is_ok());

        // Second time fails
        assert!(matches!(
            detector.check_message("msg1", now, 123),
            Err(ReplayError::ReplayDetected)
        ));
    }

    #[test]
    fn test_different_nonces_ok() {
        let detector = ReplayDetector::new(100, 300);
        let now = Utc::now().timestamp() as u64;

        assert!(detector.check_message("msg1", now, 123).is_ok());
        assert!(detector.check_message("msg1", now, 124).is_ok());
    }
}
