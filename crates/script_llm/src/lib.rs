use anyhow::Result;
use openai_api_rs::v1::{
    api::OpenAIClient,
    chat_completion::{
        self, ChatCompletionMessage, ChatCompletionRequest, Content, Tool, ToolType,
    },
    common::GPT4_O_MINI,
    message::{self, CreateMessageRequest},
    run::CreateRunRequest,
    thread::CreateThreadRequest,
    types::Function,
};
use std::time::Duration;

fn create_client(open_ai_api_key: String) -> Result<OpenAIClient> {
    let client = OpenAIClient::builder()
        .with_api_key(open_ai_api_key)
        .build()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    Ok(client)
}

fn try_into_function_tool(function: serde_json::Value) -> Result<Tool> {
    let serde_json::Value::Object(function) = function else {
        anyhow::bail!("Invalid function")
    };
    let name = function
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No name"))?;
    let description = function
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let parameters = function
        .get("parameters")
        .ok_or_else(|| anyhow::anyhow!("No parameters"))?
        .clone();
    Ok(Tool {
        r#type: ToolType::Function,
        function: Function {
            name,
            description,
            parameters: serde_json::from_value(parameters)?,
        },
    })
}

pub async fn chat_completion(open_ai_api_key: String, prompt: String) -> Result<String> {
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
    let message = &result.choices[0].message;
    let content = message.content.clone().unwrap_or_default();
    Ok(content)
}

pub async fn function_calling(
    open_ai_api_key: String,
    prompt: String,
    function: serde_json::Value,
) -> Result<serde_json::Value> {
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
    )
    .tool_choice(chat_completion::ToolChoiceType::Required)
    .tools(vec![try_into_function_tool(function)?]);

    let result = client.chat_completion(req).await?;
    let message = &result.choices[0].message;
    tracing::info!("{:#?}", message);

    let Some(tool_calls) = &message.tool_calls else {
        anyhow::bail!("No tool calls")
    };
    let Some(arguments) = &tool_calls[0].function.arguments else {
        anyhow::bail!("No arguments")
    };
    let arguments = serde_json::from_str(arguments)?;
    Ok(arguments)
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
    open_ai_api_key: String,
    thread_id: String,
    assistant_id: String,
    prompt: String,
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
    for (i, data) in results.data.iter().enumerate() {
        for (j, content) in data.content.iter().enumerate() {
            tracing::debug!(
                "[{}:{}] {:?}: {:?} {:?}",
                i,
                j,
                data.role,
                content.text.value,
                content.text.annotations
            );
        }
    }

    let response = results
        .data
        .iter()
        .find(|d| d.role == message::MessageRole::assistant)
        .ok_or_else(|| anyhow::anyhow!("No response found"))?;
    let content = response
        .content
        .first()
        .ok_or_else(|| anyhow::anyhow!("No content found"))?;
    Ok(content.text.value.clone())
}
