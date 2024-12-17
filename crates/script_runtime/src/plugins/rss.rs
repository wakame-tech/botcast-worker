use crate::{
    libs::rss::RssFeed,
    plugins::{as_string, evaluate_args},
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use rss::Channel;
use std::str::FromStr;
use tracing::instrument;

use super::Plugin;

#[derive(Clone)]
pub(crate) struct Rss;

#[async_trait::async_trait]
impl AsyncCallable for Rss {
    #[instrument(skip(self, ctx), ret)]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let url = as_string(&evaluated[0])?;
        let channel = Channel::from_str(&url)?;
        let feed: RssFeed = TryFrom::try_from(channel)?;
        let ret = serde_json::to_value(feed)?;
        Ok(ret.into())
    }
}

pub(crate) struct RssPlugin;

impl Plugin for RssPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        {
            let (name, f) = ("rss", Box::new(Rss) as Box<dyn AsyncCallable>);
            context.insert(name, Value::Function(Function::new(name, f)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::fetch::FetchPlugin;

    #[tokio::test]
    async fn call_rss() {
        let mut context = Context::new();
        FetchPlugin::default().register_functions(&mut context);
        RssPlugin.register_functions(&mut context);

        let template = serde_json::json!({
            "$eval": "rss(fetch('https://zenn.dev/feed'))"
        });
        let ret = json_e::render_with_context(&template, &context).await;
        assert!(ret.is_ok());
    }
}
