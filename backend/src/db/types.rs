#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "api_type")]
#[serde(rename_all = "snake_case")]
pub enum ApiType {
    #[sqlx(rename = "openai_chat_completions")]
    #[serde(rename = "openai_chat_completions")]
    OpenAiChatCompletions,
    #[sqlx(rename = "openai_responses")]
    #[serde(rename = "openai_responses")]
    OpenAiResponses,
    #[sqlx(rename = "openai_models")]
    #[serde(rename = "openai_models")]
    OpenAiModels,
    #[sqlx(rename = "anthropic_messages")]
    #[serde(rename = "anthropic_messages")]
    AnthropicMessages,
}

impl std::fmt::Display for ApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAiChatCompletions => write!(f, "openai_chat_completions"),
            Self::OpenAiResponses => write!(f, "openai_responses"),
            Self::OpenAiModels => write!(f, "openai_models"),
            Self::AnthropicMessages => write!(f, "anthropic_messages"),
        }
    }
}

impl std::str::FromStr for ApiType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openai_chat_completions" => Ok(Self::OpenAiChatCompletions),
            "openai_responses" => Ok(Self::OpenAiResponses),
            "openai_models" => Ok(Self::OpenAiModels),
            "anthropic_messages" => Ok(Self::AnthropicMessages),
            _ => Err(anyhow::anyhow!("unknown api_type: {}", s)),
        }
    }
}
