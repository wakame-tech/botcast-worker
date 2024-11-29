use crate::xq::run_xq;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use tracing::instrument;

#[derive(Clone)]
pub(crate) struct Jq;

#[async_trait::async_trait]
impl AsyncCallable for Jq {
    #[instrument(skip(self))]
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let (value, query) = match args {
            [value, Value::String(query)] => Ok((value.try_into()?, query)),
            _ => Err(anyhow::anyhow!("invalid arguments")),
        }?;
        let result = run_xq(query, value)?;
        Ok(result.into())
    }
}
