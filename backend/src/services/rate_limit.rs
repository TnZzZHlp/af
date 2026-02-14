use std::{collections::HashMap, time::Instant};

use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::db::gateway_keys;

#[derive(Debug, Clone)]
struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(limit: i32, window_secs: f64) -> Self {
        let capacity = limit.max(0) as f64;
        let refill_per_sec = if window_secs > 0.0 {
            capacity / window_secs
        } else {
            0.0
        };
        Self {
            capacity,
            tokens: capacity,
            refill_per_sec,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            let added = elapsed * self.refill_per_sec;
            self.tokens = (self.tokens + added).min(self.capacity);
            self.last_refill = now;
        }
    }

    fn can_consume(&self, amount: f64) -> bool {
        self.tokens >= amount
    }

    fn consume(&mut self, amount: f64) {
        self.tokens = (self.tokens - amount).max(0.0);
    }

    fn update_limit(&mut self, limit: i32, window_secs: f64) {
        let capacity = limit.max(0) as f64;
        let refill_per_sec = if window_secs > 0.0 {
            capacity / window_secs
        } else {
            0.0
        };
        self.capacity = capacity;
        self.refill_per_sec = refill_per_sec;
        self.tokens = self.tokens.min(self.capacity);
    }
}

#[derive(Debug, Clone)]
struct KeyBuckets {
    rps: Option<TokenBucket>,
    rpm: Option<TokenBucket>,
}

impl KeyBuckets {
    fn new() -> Self {
        Self {
            rps: None,
            rpm: None,
        }
    }

    fn sync_limits(&mut self, rps: Option<i32>, rpm: Option<i32>) {
        match sanitize_limit(rps) {
            Some(limit) => match &mut self.rps {
                Some(bucket) => bucket.update_limit(limit, 1.0),
                None => self.rps = Some(TokenBucket::new(limit, 1.0)),
            },
            None => self.rps = None,
        }

        match sanitize_limit(rpm) {
            Some(limit) => match &mut self.rpm {
                Some(bucket) => bucket.update_limit(limit, 60.0),
                None => self.rpm = Some(TokenBucket::new(limit, 60.0)),
            },
            None => self.rpm = None,
        }
    }

    fn allow_and_consume(&mut self, now: Instant) -> bool {
        if let Some(bucket) = &mut self.rps {
            bucket.refill(now);
            if !bucket.can_consume(1.0) {
                return false;
            }
        }

        if let Some(bucket) = &mut self.rpm {
            bucket.refill(now);
            if !bucket.can_consume(1.0) {
                return false;
            }
        }

        if let Some(bucket) = &mut self.rps {
            bucket.consume(1.0);
        }

        if let Some(bucket) = &mut self.rpm {
            bucket.consume(1.0);
        }

        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct RateLimiter {
    buckets: std::sync::Arc<Mutex<HashMap<Uuid, KeyBuckets>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn check_and_consume(
        &self,
        gateway_key_id: Uuid,
        rps: Option<i32>,
        rpm: Option<i32>,
    ) -> bool {
        if sanitize_limit(rps).is_none() && sanitize_limit(rpm).is_none() {
            return true;
        }

        let mut guard = self.buckets.lock().await;
        let entry = guard.entry(gateway_key_id).or_insert_with(KeyBuckets::new);
        entry.sync_limits(rps, rpm);
        entry.allow_and_consume(Instant::now())
    }
}

fn sanitize_limit(limit: Option<i32>) -> Option<i32> {
    match limit {
        Some(value) if value > 0 => Some(value),
        _ => None,
    }
}

pub async fn fetch_limits(
    pool: &PgPool,
    gateway_key_id: Uuid,
) -> anyhow::Result<(Option<i32>, Option<i32>)> {
    gateway_keys::fetch_limits(pool, gateway_key_id).await
}
