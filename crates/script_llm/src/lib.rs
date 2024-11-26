use anyhow::Result;
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{self, ChatCompletionMessage, ChatCompletionRequest, Content},
    common::GPT4_O_MINI,
    message::{self, CreateMessageRequest},
    run::CreateRunRequest,
    thread::CreateThreadRequest,
};
use std::time::Duration;

fn create_client(open_ai_api_key: String) -> Result<OpenAIClient> {
    let client = OpenAIClient::builder()
        .with_api_key(open_ai_api_key)
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(client)
}

pub async fn chat_completion(prompt: String, open_ai_api_key: String) -> Result<String> {
    let client = create_client(open_ai_api_key)?;
    let req = ChatCompletionRequest::new(
        GPT4_O_MINI.to_string(),
        vec![ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: Content::Text(prompt.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
    );

    let result = client.chat_completion(req).await?;
    let content = result.choices[0]
        .message
        .content
        .clone()
        .unwrap_or_default();
    Ok(content)
}

pub async fn create_thread(open_ai_api_key: String) -> Result<String> {
    let client = create_client(open_ai_api_key)?;
    let req = CreateThreadRequest::new();
    let result = client.create_thread(req).await?;
    Ok(result.id)
}

pub async fn delete_thread(open_ai_api_key: String, thread_id: String) -> Result<()> {
    let client = create_client(open_ai_api_key)?;
    client.delete_thread(thread_id).await?;
    Ok(())
}

pub async fn chat_assistant(
    prompt: String,
    thread_id: String,
    assistant_id: String,
    open_ai_api_key: String,
) -> Result<String> {
    let client = create_client(open_ai_api_key)?;
    let req = CreateMessageRequest::new(message::MessageRole::user, prompt);
    let _ = client.create_message(thread_id.clone(), req).await?;

    let run = client
        .create_run(thread_id.clone(), CreateRunRequest::new(assistant_id))
        .await?;
    loop {
        let res = client
            .retrieve_run(thread_id.clone(), run.id.clone())
            .await?;
        if res.status == "completed" {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let results = client.list_messages(thread_id).await?;
    let response = results
        .data
        .iter()
        .filter(|d| d.role == message::MessageRole::assistant)
        .last()
        .ok_or_else(|| anyhow::anyhow!("No response found"))?;
    let content = response
        .content
        .iter()
        .last()
        .ok_or_else(|| anyhow::anyhow!("No content found"))?;
    Ok(content.text.value.clone())
}
