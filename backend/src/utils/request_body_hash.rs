pub type RequestBodyHash = [u8; 16];

pub fn hash_request_body(request_body: &[u8]) -> RequestBodyHash {
    if let Some(bytes) = canonical_json_bytes(request_body) {
        md5::compute(bytes).0
    } else {
        md5::compute(request_body).0
    }
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

pub fn hash_request_body_hex(request_body: &[u8]) -> String {
    request_body_hash_hex(hash_request_body(request_body))
}

fn canonical_json_bytes(request_body: &[u8]) -> Option<Vec<u8>> {
    let value: serde_json::Value = serde_json::from_slice(request_body).ok()?;
    serde_json::to_vec(&value).ok()
}
