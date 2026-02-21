use serde_json::Value;

use crate::error::AppError;

pub mod request_body_hash;

pub fn extract_model_from_payload(payload: &Value) -> Result<String, AppError> {
    payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| AppError::BadRequest("model is required".to_string()))
}
