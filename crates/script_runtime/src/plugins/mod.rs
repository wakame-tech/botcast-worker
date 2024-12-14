pub mod botcast_api;
mod eval;
mod fetch;
mod html;
mod json;
mod llm;
mod rss;
mod time;

use anyhow::Result;
use futures::future::try_join_all;
use json_e::{render_with_context, value::Value, Context};

pub trait Plugin {
    fn register_functions(&self, context: &mut Context<'_>);
}

pub(crate) fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(html::HtmlPlugin),
        Box::new(json::JsonPlugin),
        Box::new(rss::RssPlugin),
        Box::new(time::TimePlugin),
        Box::new(fetch::FetchPlugin::default()),
        Box::new(llm::LlmPlugin),
        Box::new(eval::EvalPlugin),
    ]
}

async fn evaluate_args<'a>(
    ctx: &'_ Context<'_>,
    args: &'a [Value],
) -> Result<Vec<serde_json::Value>> {
    let args: Vec<serde_json::Value> = args
        .iter()
        .map(|v| v.try_into())
        .collect::<Result<Vec<_>>>()?;
    try_join_all(args.iter().map(|v| render_with_context(v, ctx))).await
}

fn as_string(value: &serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Err(anyhow::anyhow!("expected a string")),
    }
}
