use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{value::Value, Context};
use readable_text::ReadableText;
use script_http_client::HttpClient;

fn http_client() -> HttpClient {
    HttpClient::new(std::env::var("USER_AGENT").ok())
}

pub fn fetch<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [Value::String(url)] => {
                let html = http_client().fetch_content_as_utf8(url.clone()).await?;
                Ok(Value::String(html))
            }
            _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
        }
    }
    .boxed()
}

pub fn text<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [Value::String(html)] => {
                let md = ReadableText::extract(html)?;
                Ok(Value::String(md))
            }
            _ => Err(anyhow::anyhow!("failed to extract".to_string())),
        }
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use crate::imports::define_imports;

    #[tokio::test]
    async fn test_call_fetch() {
        std::env::set_var("USER_AGENT", "mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/127.0.0.0 safari/537.36");
        let mut context = json_e::Context::new();
        define_imports(&mut context);

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
