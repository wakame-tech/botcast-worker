use crate::{runtime::evaluate_args, xq::run_xq};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use tracing::instrument;

use super::as_string;

#[derive(Clone)]
pub(crate) struct Jq;

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
pub(crate) struct Hq;

#[async_trait::async_trait]
impl AsyncCallable for Hq {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, &args).await?;
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
pub(crate) struct Replace;

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
