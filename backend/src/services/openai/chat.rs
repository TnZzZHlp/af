use axum::{body::Body, http::Response};
use serde_json::Value;

use crate::{
    db::types::ApiType,
    error::AppResult,
    middleware::{auth::GatewayKeyId, request_log::ClientInfo},
};

use super::OpenAiService;

impl OpenAiService {
    pub async fn chat_completions(
        &self,
        gateway_key_id: GatewayKeyId,
        payload: Value,
        client_info: ClientInfo,
    ) -> AppResult<Response<Body>> {
        self.process_request(
            gateway_key_id,
            payload,
            client_info,
            ApiType::OpenAiChatCompletions,
        )
        .await
    }
}
