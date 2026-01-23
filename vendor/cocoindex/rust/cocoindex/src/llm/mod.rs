use crate::prelude::*;

use crate::base::json_schema::ToJsonSchemaOptions;

use schemars::schema::SchemaObject;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LlmApiType {
    Ollama,
    OpenAi,
    Gemini,
    Anthropic,
    LiteLlm,
    OpenRouter,
    Voyage,
    Vllm,
    VertexAi,
    Bedrock,
    AzureOpenAi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAiConfig {
    pub project: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAiConfig {
    pub deployment_id: String,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LlmApiConfig {
    VertexAi(VertexAiConfig),
    OpenAi(OpenAiConfig),
    AzureOpenAi(AzureOpenAiConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSpec {
    pub api_type: LlmApiType,
    pub address: Option<String>,
    pub model: String,
    pub api_key: Option<spec::AuthEntryReference<String>>,
    pub api_config: Option<LlmApiConfig>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum OutputFormat<'a> {
    JsonSchema {
        name: Cow<'a, str>,
        schema: Cow<'a, SchemaObject>,
    },
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LlmGenerateRequest<'a> {
    pub model: &'a str,
    pub system_prompt: Option<Cow<'a, str>>,
    pub user_prompt: Cow<'a, str>,
    pub image: Option<Cow<'a, [u8]>>,
    pub output_format: Option<OutputFormat<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum GeneratedOutput {
    Json(serde_json::Value),
    Text(String),
}

#[derive(Debug)]
pub struct LlmGenerateResponse {
    pub output: GeneratedOutput,
}

#[async_trait]
pub trait LlmGenerationClient: Send + Sync {
    async fn generate<'req>(
        &self,
        request: LlmGenerateRequest<'req>,
    ) -> Result<LlmGenerateResponse>;

    fn json_schema_options(&self) -> ToJsonSchemaOptions;
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LlmEmbeddingRequest<'a> {
    pub model: &'a str,
    pub texts: Vec<Cow<'a, str>>,
    pub output_dimension: Option<u32>,
    pub task_type: Option<Cow<'a, str>>,
}

pub struct LlmEmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}

#[async_trait]
pub trait LlmEmbeddingClient: Send + Sync {
    async fn embed_text<'req>(
        &self,
        request: LlmEmbeddingRequest<'req>,
    ) -> Result<LlmEmbeddingResponse>;

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32>;

    fn behavior_version(&self) -> Option<u32> {
        Some(1)
    }
}

// mod anthropic;
// mod bedrock;
// mod gemini;
// mod litellm;
// mod ollama;
// mod openai;
// mod openrouter;
// mod vllm;
// mod voyage;

pub async fn new_llm_generation_client(
    _api_type: LlmApiType,
    _address: Option<String>,
    _api_key: Option<String>,
    _api_config: Option<LlmApiConfig>,
) -> Result<Box<dyn LlmGenerationClient>> {
    api_bail!("LLM support is disabled in this build")
}

pub async fn new_llm_embedding_client(
    _api_type: LlmApiType,
    _address: Option<String>,
    _api_key: Option<String>,
    _api_config: Option<LlmApiConfig>,
) -> Result<Box<dyn LlmEmbeddingClient>> {
    api_bail!("LLM support is disabled in this build")
}
