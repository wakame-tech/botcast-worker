use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use readable_text::ReadableText;
use script_http_client::HttpClient;

fn http_client() -> HttpClient {
    HttpClient::new(std::env::var("USER_AGENT").ok())
}

#[derive(Clone)]
pub(crate) struct Fetch;

#[async_trait::async_trait]
impl AsyncCallable for Fetch {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [Value::String(url)] => {
                let html = http_client().fetch_content_as_utf8(url.clone()).await?;
                Ok(Value::String(html))
            }
            _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct FetchJson;

#[async_trait::async_trait]
impl AsyncCallable for FetchJson {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [Value::String(url)] => {
                let json = http_client().fetch_json(url.clone()).await?;
                Ok(json.into())
            }
            _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct Text;

#[async_trait::async_trait]
impl AsyncCallable for Text {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [Value::String(html)] => {
                let md = ReadableText::extract(html)?;
                Ok(Value::String(md))
            }
            _ => Err(anyhow::anyhow!("failed to extract".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::imports::create_context;
    use json_e::Context;

    #[tokio::test]
    async fn test_call_fetch() {
        std::env::set_var("USER_AGENT", "mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/127.0.0.0 safari/537.36");
        let mut context = Context::new();
        create_context(&mut context);
        let template = serde_json::json!({
            "$let": {
                "html": { "$eval": "fetch('https://www.aozora.gr.jp/cards/000081/files/45630_23908.html')" },
            },
            "in": {
                "$eval": "text(html)",
            }
        });
        let result = json_e::render_with_context(&template, &context)
            .await
            .unwrap();
        assert!(result.is_string());
    }
}
