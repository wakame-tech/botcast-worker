use crate::Urn;
use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{
    value::{Function, Value},
    Context,
};
use readable_text::ReadableText;
use script_http_client::HttpClient;

pub(crate) struct Imports {
    http_client: HttpClient,
}

impl Default for Imports {
    fn default() -> Self {
        Self {
            http_client: HttpClient::new(std::env::var("USER_AGENT").ok()),
        }
    }
}

impl Imports {
    pub(crate) async fn today(&self, format: String) -> Result<String> {
        Ok(chrono::Local::now().format(&format).to_string())
    }

    pub(crate) async fn get(&self, urn: Urn) -> Result<String> {
        // TODO
        Ok(urn.0)
    }

    pub(crate) async fn fetch(&self, url: String) -> Result<String> {
        let html = self.http_client.fetch_content_as_utf8(url).await?;
        Ok(html)
    }

    pub(crate) async fn text(&self, html: String) -> Result<String> {
        ReadableText::extract(&html)
    }

    pub(crate) async fn llm(&self, _prompt: String) -> Result<String> {
        // TODO
        Ok("llm".to_string())
    }
}

fn today<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports::default();
        match args {
            [Value::String(format)] => {
                let today = imports.today(format.clone()).await?;
                Ok(Value::String(today))
            }
            _ => Err(anyhow::anyhow!("today only supports a string".to_string())),
        }
    }
    .boxed()
}

fn get<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports::default();
        match args {
            [Value::String(urn)] => {
                let resource = imports.get(Urn(urn.clone())).await?;
                Ok(Value::String(resource))
            }
            _ => Err(anyhow::anyhow!("get only supports a string".to_string())),
        }
    }
    .boxed()
}

fn fetch<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports::default();
        match args {
            [Value::String(url)] => {
                let html = imports.fetch(url.clone()).await?;
                Ok(Value::String(html))
            }
            _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
        }
    }
    .boxed()
}

fn text<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports::default();
        match args {
            [Value::String(html)] => {
                let md = imports.text(html.clone()).await?;
                Ok(Value::String(md))
            }
            _ => Err(anyhow::anyhow!("failed to extract".to_string())),
        }
    }
    .boxed()
}

fn llm<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports::default();
        match args {
            [Value::String(prompt)] => {
                let res = imports.llm(prompt.clone()).await?;
                Ok(Value::String(res))
            }
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }
    }
    .boxed()
}

pub(crate) fn define_imports<'a>(context: &mut Context<'a>) {
    // TODO: to be Imports as trait, and use it
    context.insert("today", Value::Function(Function::new("today", today)));
    context.insert("get", Value::Function(Function::new("get", get)));
    context.insert("fetch", Value::Function(Function::new("fetch", fetch)));
    context.insert("text", Value::Function(Function::new("text", text)));
    context.insert("llm", Value::Function(Function::new("llm", llm)));
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_call_today() {
        let mut context = json_e::Context::new();
        super::define_imports(&mut context);
        let result = json_e::render_with_context(
            &serde_json::json!({ "$eval": "today('%Y/%m/%d')" }),
            &context,
        )
        .await
        .unwrap();
        let expected = serde_json::json!(chrono::Local::now().format("%Y/%m/%d").to_string());
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_call_fetch() {
        std::env::set_var("USER_AGENT", "mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/127.0.0.0 safari/537.36");
        let mut context = json_e::Context::new();
        super::define_imports(&mut context);

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
