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
    #[sqlx(rename = "anthropic_messages")]
    #[serde(rename = "anthropic_messages")]
    AnthropicMessages,
}

impl std::fmt::Display for ApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAiChatCompletions => write!(f, "openai_chat_completions"),
            Self::OpenAiResponses => write!(f, "openai_responses"),
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
            "anthropic_messages" => Ok(Self::AnthropicMessages),
            _ => Err(anyhow::anyhow!("unknown api_type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "lb_strategy")]
#[serde(rename_all = "snake_case")]
pub enum LbStrategy {
    #[sqlx(rename = "weighted_round_robin")]
    #[serde(rename = "weighted_round_robin")]
    WeightedRoundRobin,
    #[sqlx(rename = "round_robin")]
    #[serde(rename = "round_robin")]
    RoundRobin,
    #[sqlx(rename = "priority")]
    #[serde(rename = "priority")]
    Priority,
}

impl std::fmt::Display for LbStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WeightedRoundRobin => write!(f, "weighted_round_robin"),
            Self::RoundRobin => write!(f, "round_robin"),
            Self::Priority => write!(f, "priority"),
        }
    }
}

impl std::str::FromStr for LbStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "weighted_round_robin" => Ok(Self::WeightedRoundRobin),
            "round_robin" => Ok(Self::RoundRobin),
            "priority" => Ok(Self::Priority),
            _ => Err(anyhow::anyhow!("unknown lb_strategy: {}", s)),
        }
    }
}
