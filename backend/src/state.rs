use crate::services::{
    auth::LoginProtection, background::BackgroundTasks, openai::OpenAiService,
    rate_limit::RateLimiter, response_cache::ResponseCache,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub openai: OpenAiService,
    pub jwt_secret: String,
    pub rate_limiter: RateLimiter,
    pub login_protection: LoginProtection,
    pub response_cache: ResponseCache,
    pub background_tasks: BackgroundTasks,
}
