use crate::db::request_logs::CachedResponse;
use moka::sync::Cache;

pub use crate::utils::request_body_hash::{
    RequestBodyHash, hash_request_body, request_body_hash_hex,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResponseCacheKey {
    pub request_body_hash: RequestBodyHash,
}

impl ResponseCacheKey {
    pub fn new(request_body: &[u8]) -> Self {
        Self {
            request_body_hash: hash_request_body(request_body),
        }
    }
}

#[derive(Clone)]
pub struct ResponseCache {
    inner: Cache<ResponseCacheKey, CachedResponse>,
}

impl ResponseCache {
    pub fn new() -> Self {
        let inner = Cache::builder().max_capacity(1000).build();

        Self { inner }
    }

    pub fn get(&self, key: &ResponseCacheKey) -> Option<CachedResponse> {
        self.inner.get(key)
    }

    pub fn insert(&self, key: ResponseCacheKey, value: CachedResponse) {
        self.inner.insert(key, value);
    }
}
