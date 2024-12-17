use super::{as_string, evaluate_args, Plugin};
use crate::libs::xq::run_xq;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use tracing::instrument;

#[derive(Clone)]
struct Jq;

#[async_trait::async_trait]
impl AsyncCallable for Jq {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let value = args[0].clone();
        let query = as_string(&args[1])?;

        let ret = run_xq(&query, value)?;
        Ok(ret.into())
    }
}

#[derive(Clone)]
struct Hq;

#[async_trait::async_trait]
impl AsyncCallable for Hq {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let html = as_string(&args[0])?;
        let query = as_string(&args[1])?;

        let dom = tl::parse(&html, tl::ParserOptions::default())?;
        let parser = dom.parser();
        let element = dom
            .query_selector(&query)
            .ok_or(anyhow::anyhow!("query failed"))?
            .next()
            .ok_or(anyhow::anyhow!("query failed"))?
            .get(parser)
            .ok_or(anyhow::anyhow!("query failed"))?;
        let ret = element.inner_html(parser).to_string();
        Ok(Value::String(ret))
    }
}

#[derive(Clone)]
struct Replace;

#[async_trait::async_trait]
impl AsyncCallable for Replace {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let text = as_string(&args[0])?;
        let pat = as_string(&args[1])?;
        let to = as_string(&args[2])?;

        let ret = text.replace(&pat, &to);
        Ok(Value::String(ret))
    }
}

pub(crate) struct JsonPlugin;

impl Plugin for JsonPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        for (name, function) in [
            ("jq", Box::new(Jq) as Box<dyn AsyncCallable>),
            ("hq", Box::new(Hq)),
            ("replace", Box::new(Replace)),
        ] {
            context.insert(name, Value::Function(Function::new(name, function)));
        }
    }
}
