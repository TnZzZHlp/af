use crate::services::openai::OpenAiService;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub openai: OpenAiService,
}
