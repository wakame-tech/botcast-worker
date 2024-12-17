use super::{as_string, evaluate_args, Plugin};
use crate::libs::http_client::HttpClient;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
struct Fetch {
    client: Arc<HttpClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for Fetch {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let url = as_string(&args[0])?;
        let html = self.client.fetch_content_as_utf8(url.clone()).await?;
        Ok(Value::String(html))
    }
}

#[derive(Clone)]
struct FetchJson {
    client: Arc<HttpClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for FetchJson {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let url = as_string(&evaluated[0])?;
        let json = self.client.fetch_json(url.clone()).await?;
        Ok(json.into())
    }
}

#[derive(Default)]
pub struct FetchPlugin {
    client: Arc<HttpClient>,
}

impl Plugin for FetchPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        for (name, f) in [
            (
                "fetch",
                Box::new(Fetch {
                    client: self.client.clone(),
                }) as Box<dyn AsyncCallable>,
            ),
            (
                "fetch_json",
                Box::new(FetchJson {
                    client: self.client.clone(),
                }),
            ),
        ] {
            context.insert(name, Value::Function(Function::new(name, f)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json_e::Context;

    #[tokio::test]
    async fn test_call_fetch() {
        std::env::set_var("USER_AGENT", "mozilla/5.0 (x11; linux x86_64) applewebkit/537.36 (khtml, like gecko) chrome/127.0.0.0 safari/537.36");
        let mut context = Context::new();
        FetchPlugin::default().register_functions(&mut context);

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
