use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::alias_targets::{self, AliasTargetDetail};
use crate::db::provider_keys::{self, ProviderKey};
use crate::db::types::ApiType;
use crate::error::AppError;
use crate::services::{auth, providers};

#[derive(Debug, Clone)]
pub struct Route {
    pub provider_id: Uuid,
    pub provider_name: String,
    pub endpoint_url: String,
    pub model_id: String,
    pub provider_key: ProviderKey,
    pub alias_name: String,
}

pub async fn resolve_route(
    pool: &PgPool,
    gateway_key_id: Uuid,
    model: &str,
    api_type: ApiType,
) -> anyhow::Result<Route> {
    // 1. Check whitelist
    enforce_model_whitelist(pool, gateway_key_id, model).await?;

    // 2. Resolve target
    tracing::debug!("resolving target");
    let mut targets = Vec::new();
    let mut is_alias_match = false;

    if let Some((brief, real_model)) = parse_provider_real_model(model) {
        tracing::debug!(
            brief,
            real_model,
            "detected potential provider brief in model name"
        );
        if let Ok(Some(provider)) = providers::get_provider_by_brief(pool, brief).await {
            tracing::debug!(
                provider_id = %provider.id,
                provider_name = %provider.name,
                "found provider by brief"
            );
            let endpoints = providers::list_endpoints_by_provider(pool, provider.id)
                .await
                .unwrap_or_default();

            if let Some(endpoint) = endpoints
                .into_iter()
                .find(|e| e.api_type == api_type && e.enabled)
            {
                tracing::debug!(endpoint_id = %endpoint.id, url = %endpoint.url, "found suitable endpoint");
                targets.push(AliasTargetDetail {
                    id: Uuid::now_v7(),
                    alias_id: Uuid::now_v7(),
                    alias_name: model.to_string(),
                    alias_target_id: Uuid::now_v7(),
                    provider_id: provider.id,
                    provider_name: provider.name,
                    provider_usage_count: provider.usage_count,
                    usage_count: 0,
                    provider_endpoint_id: Some(endpoint.id),
                    endpoint_url: Some(endpoint.url),
                    model_id: real_model.to_string(),
                    enabled: true,
                    created_at: OffsetDateTime::now_utc(),
                });
            } else {
                tracing::debug!("no suitable endpoint found for provider");
            }
        } else {
            tracing::debug!("no provider found by brief");
        }
    }

    if targets.is_empty() {
        tracing::debug!("fetching alias target details");
        targets = fetch_alias_target_details(pool, model, api_type).await?;
        is_alias_match = true;
    }

    let target = targets
        .first()
        .ok_or_else(|| AppError::BadRequest(format!("unknown model alias: {model}")))?;

    tracing::debug!(
        provider_id = %target.provider_id,
        provider_name = %target.provider_name,
        target_model_id = %target.model_id,
        "resolved alias target"
    );

    if is_alias_match {
        tracing::debug!(alias_target_id = %target.alias_target_id, "incrementing alias target usage count");
        alias_targets::increment_usage_count(pool, target.alias_target_id).await?;
    }

    // 3. Fetch provider keys
    tracing::debug!("fetching provider keys");
    let provider_keys = fetch_provider_keys(pool, target.provider_id).await?;

    let provider_key = provider_keys
        .first()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no provider keys available")))?
        .clone();

    tracing::debug!(provider_key_id = %provider_key.id, "selected provider key");

    let endpoint_url = target.endpoint_url.clone().ok_or_else(|| {
        AppError::Internal(anyhow::anyhow!("no endpoint url found for target provider"))
    })?;

    Ok(Route {
        provider_id: target.provider_id,
        provider_name: target.provider_name.clone(),
        endpoint_url,
        model_id: target.model_id.clone(),
        provider_key,
        alias_name: model.to_string(),
    })
}

async fn enforce_model_whitelist(
    pool: &PgPool,
    gateway_key_id: Uuid,
    model: &str,
) -> anyhow::Result<()> {
    let whitelist = auth::fetch_model_whitelist(pool, gateway_key_id).await?;
    if whitelist.is_empty() {
        tracing::debug!("whitelist is empty, skipping check");
        return Ok(());
    }

    tracing::debug!(?whitelist, %model, "checking model whitelist");
    if whitelist.iter().any(|entry| entry == model) {
        return Ok(());
    }

    tracing::debug!("model not in whitelist");
    Err(AppError::Forbidden("model not in whitelist".to_string()).into())
}

fn parse_provider_real_model(model: &str) -> Option<(&str, &str)> {
    let (brief, real_model) = model.split_once(':')?;
    let brief = brief.trim();
    let real_model = real_model.trim();
    if brief.is_empty() || real_model.is_empty() {
        return None;
    }
    Some((brief, real_model))
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: ApiType,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = alias_targets::fetch_alias_target_details(pool, alias_name, api_type).await?;

    Ok(rows)
}

pub async fn fetch_provider_keys(
    pool: &PgPool,
    provider_id: Uuid,
) -> anyhow::Result<Vec<ProviderKey>> {
    let rows = provider_keys::fetch_provider_keys(pool, provider_id).await?;

    Ok(rows)
}
