use crate::resolve::resolve_urn;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::{provider::Provider, urn::Urn};

#[derive(Clone)]
pub(crate) struct Eval;

#[async_trait::async_trait]
impl AsyncCallable for Eval {
    async fn call(&self, context: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [template, Value::Object(values)] => {
                let mut context = context.child();
                for (k, v) in values.iter() {
                    context.insert(k, v.clone());
                }
                let template = template.try_into()?;
                let evaluated = json_e::render_with_context(&template, &context).await?;
                Ok(evaluated.into())
            }
            _ => Err(anyhow::anyhow!("invalid args".to_string())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct UrnGet;

#[async_trait::async_trait]
impl AsyncCallable for UrnGet {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let urn = match args {
            [Value::String(urn)] => urn.parse::<Urn>(),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        }?;
        let value = resolve_urn(Provider, urn).await?;
        Ok(value.into())
    }
}
