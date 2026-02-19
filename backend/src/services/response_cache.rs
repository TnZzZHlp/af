use crate::db::request_logs::CachedResponse;
use moka::sync::Cache;

pub type RequestBodyHash = [u8; 16];

pub fn hash_request_body(request_body: &[u8]) -> RequestBodyHash {
    md5::compute(request_body).0
}

pub fn request_body_hash_hex(hash: RequestBodyHash) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(hash.len() * 2);
    for byte in hash {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

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
