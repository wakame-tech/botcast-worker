use crate::{
    imports::{display_fn_io, insert_values},
    provider::DefaultProvider,
    resolve::resolve_urn,
};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::urn::Urn;

#[derive(Clone)]
pub(crate) struct Eval;

#[async_trait::async_trait]
impl AsyncCallable for Eval {
    async fn call(&self, context: &Context<'_>, args: &[Value]) -> Result<Value> {
        let (template, values) = match args {
            [template, Value::Object(values)] => (template, values),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        };

        let mut context = context.child();
        insert_values(&mut context, values.clone());
        let template = template.try_into()?;
        let ret = json_e::render_with_context(&template, &context)
            .await
            .map(Value::from)
            .map_err(|e| anyhow::anyhow!("eval error: {}", e))
            .and_then(|v| v.try_into());
        log::info!("{}", display_fn_io("eval", args, &ret)?);
        Ok(ret?.into())
    }
}

#[derive(Clone)]
pub(crate) struct UrnGet {
    pub(crate) provider: DefaultProvider,
}

#[async_trait::async_trait]
impl AsyncCallable for UrnGet {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let urn = match args {
            [Value::String(urn)] => urn.parse::<Urn>(),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        }?;
        let value = resolve_urn(self.provider, urn).await;
        log::info!("{}", display_fn_io("get", args, &value)?);
        Ok(value?.into())
    }
}
