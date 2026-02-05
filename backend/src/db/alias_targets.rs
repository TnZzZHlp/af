use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AliasTargetRow {
    pub id: Uuid,
    pub alias_id: Uuid,
    pub provider_endpoint_id: Uuid,
    pub model_id: Uuid,
    pub weight: i32,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct AliasTargetDetail {
    pub alias_id: Uuid,
    pub alias_name: String,
    pub alias_strategy: String,
    pub alias_target_id: Uuid,
    pub target_weight: i32,
    pub target_priority: i32,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_endpoint_id: Uuid,
    pub endpoint_url: String,
    pub endpoint_timeout_ms: i32,
    pub endpoint_weight: i32,
    pub endpoint_priority: i32,
    pub model_id: Uuid,
    pub model_name: String,
}

pub async fn fetch_alias_targets(
    pool: &PgPool,
    alias_id: Uuid,
) -> anyhow::Result<Vec<AliasTargetRow>> {
    let rows = sqlx::query(
        "SELECT id, alias_id, provider_endpoint_id, model_id, weight, priority\n         FROM alias_targets\n         WHERE alias_id = $1 AND enabled = true\n         ORDER BY priority DESC, weight DESC",
    )
    .bind(alias_id)
    .fetch_all(pool)
    .await?;

    let mut targets = Vec::with_capacity(rows.len());
    for row in rows {
        targets.push(AliasTargetRow {
            id: row.try_get("id")?,
            alias_id: row.try_get("alias_id")?,
            provider_endpoint_id: row.try_get("provider_endpoint_id")?,
            model_id: row.try_get("model_id")?,
            weight: row.try_get("weight")?,
            priority: row.try_get("priority")?,
        });
    }

    Ok(targets)
}

pub async fn fetch_alias_target_details(
    pool: &PgPool,
    alias_name: &str,
    api_type: &str,
) -> anyhow::Result<Vec<AliasTargetDetail>> {
    let rows = sqlx::query(
        "SELECT\n            a.id AS alias_id,\n            a.name AS alias_name,\n            a.strategy AS alias_strategy,\n            at.id AS alias_target_id,\n            at.weight AS target_weight,\n            at.priority AS target_priority,\n            p.id AS provider_id,\n            p.name AS provider_name,\n            pe.id AS provider_endpoint_id,\n            pe.url AS endpoint_url,\n            pe.timeout_ms AS endpoint_timeout_ms,\n            pe.weight AS endpoint_weight,\n            pe.priority AS endpoint_priority,\n            m.id AS model_id,\n            m.name AS model_name\n         FROM aliases a\n         JOIN alias_targets at\n           ON at.alias_id = a.id AND at.enabled = true\n         JOIN provider_endpoints pe\n           ON pe.id = at.provider_endpoint_id AND pe.enabled = true\n         JOIN providers p\n           ON p.id = pe.provider_id AND p.enabled = true\n         JOIN models m\n           ON m.id = at.model_id AND m.enabled = true\n         WHERE a.name = $1 AND a.api_type = $2 AND a.enabled = true\n         ORDER BY at.priority DESC, at.weight DESC",
    )
    .bind(alias_name)
    .bind(api_type)
    .fetch_all(pool)
    .await?;

    let mut details = Vec::with_capacity(rows.len());
    for row in rows {
        details.push(AliasTargetDetail {
            alias_id: row.try_get("alias_id")?,
            alias_name: row.try_get("alias_name")?,
            alias_strategy: row.try_get("alias_strategy")?,
            alias_target_id: row.try_get("alias_target_id")?,
            target_weight: row.try_get("target_weight")?,
            target_priority: row.try_get("target_priority")?,
            provider_id: row.try_get("provider_id")?,
            provider_name: row.try_get("provider_name")?,
            provider_endpoint_id: row.try_get("provider_endpoint_id")?,
            endpoint_url: row.try_get("endpoint_url")?,
            endpoint_timeout_ms: row.try_get("endpoint_timeout_ms")?,
            endpoint_weight: row.try_get("endpoint_weight")?,
            endpoint_priority: row.try_get("endpoint_priority")?,
            model_id: row.try_get("model_id")?,
            model_name: row.try_get("model_name")?,
        });
    }

    Ok(details)
}
