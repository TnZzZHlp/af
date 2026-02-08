use crate::services::{
    auth::LoginProtection, openai::OpenAiService, rate_limit::RateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub openai: OpenAiService,
    pub jwt_secret: String,
    pub rate_limiter: RateLimiter,
    pub login_protection: LoginProtection,
}
