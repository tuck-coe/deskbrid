use std::time::Instant;

use crate::DaemonState;

const DEFAULT_RATE_LIMIT_PER_SECOND: f64 = 30.0;
const DEFAULT_RATE_LIMIT_BURST: f64 = 120.0;

#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub per_second: f64,
    pub burst: f64,
}

#[derive(Debug, Clone)]
pub struct RateBucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RateLimitHit {
    pub retry_after_ms: u64,
}

pub(crate) fn rate_limit_from_env() -> Option<RateLimitConfig> {
    let per_second = std::env::var("DESKBRID_RATE_LIMIT_PER_SEC")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT_PER_SECOND);
    if per_second <= 0.0 {
        return None;
    }

    let burst = std::env::var("DESKBRID_RATE_LIMIT_BURST")
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT_BURST)
        .max(1.0);

    Some(RateLimitConfig { per_second, burst })
}

impl RateBucket {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.burst,
            last_refill: Instant::now(),
        }
    }

    fn take(&mut self, config: RateLimitConfig) -> Option<RateLimitHit> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        self.tokens = (self.tokens + elapsed * config.per_second).min(config.burst);

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            return None;
        }

        let missing = 1.0 - self.tokens;
        let retry_after_ms = ((missing / config.per_second) * 1000.0).ceil() as u64;
        Some(RateLimitHit { retry_after_ms })
    }
}

pub(crate) async fn check_rate_limit(state: &DaemonState, peer_uid: u32) -> Option<RateLimitHit> {
    let config = state.rate_limit?;
    let mut buckets = state.rate_limits.lock().await;
    let bucket = buckets
        .entry(peer_uid)
        .or_insert_with(|| RateBucket::new(config));
    bucket.take(config)
}

pub(crate) fn rate_limited_response(seq: u64, hit: RateLimitHit) -> serde_json::Value {
    serde_json::json!({
        "type": "response",
        "id": "action",
        "seq": seq,
        "status": "error",
        "error": {
            "code": "RATE_LIMITED",
            "message": format!("rate limit exceeded; retry after {} ms", hit.retry_after_ms),
            "retry_after_ms": hit.retry_after_ms
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_allows_burst_then_reports_retry() {
        let config = RateLimitConfig {
            per_second: 1.0,
            burst: 1.0,
        };
        let mut bucket = RateBucket::new(config);

        assert!(bucket.take(config).is_none());
        let hit = bucket.take(config).expect("limited");
        assert!(hit.retry_after_ms > 0);
    }
}
