use super::as_string;
use crate::runtime::evaluate_args;
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
        let url = match args {
            [Value::String(url)] => Ok(url),
            _ => Err(anyhow::anyhow!("invalid args".to_string())),
        }?;
        let html = http_client().fetch_content_as_utf8(url.clone()).await?;
        Ok(Value::String(html))
    }
}

#[derive(Clone)]
pub(crate) struct FetchJson;

#[async_trait::async_trait]
impl AsyncCallable for FetchJson {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let url = as_string(&evaluated[0])?;
        let json = http_client().fetch_json(url.clone()).await?;
        Ok(json.into())
    }
}

#[derive(Clone)]
pub(crate) struct Text;

#[async_trait::async_trait]
impl AsyncCallable for Text {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let html = as_string(&evaluated[0])?;
        let md = ReadableText::extract(&html)?;
        Ok(Value::String(md))
    }
}

#[cfg(test)]
mod tests {
    use crate::imports::insert_custom_functions;
    use json_e::Context;

    #[tokio::test]
    async fn test_call_fetch() {
        std::env::set_var("USER_AGENT", "mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/127.0.0.0 safari/537.36");
        let mut context = Context::new();
        insert_custom_functions(&mut context);
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
