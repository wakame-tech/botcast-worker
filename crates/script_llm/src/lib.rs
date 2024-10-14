use anyhow::Result;
use langchain_rust::{
    language_models::llm::LLM,
    llm::{OpenAI, OpenAIConfig, OpenAIModel},
};

pub async fn langchain(prompt: &str) -> Result<String> {
    let open_ai_api_key = std::env::var("OPENAI_API_KEY")?;
    let open_ai = OpenAI::default()
        .with_model(OpenAIModel::Gpt4oMini)
        .with_config(OpenAIConfig::default().with_api_key(open_ai_api_key));

    let response = open_ai.invoke(prompt).await?;
    Ok(response)
}
