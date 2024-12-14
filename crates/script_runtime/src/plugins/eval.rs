use super::Plugin;
use crate::runtime::insert_values;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use tracing::instrument;

#[derive(Clone)]
struct Eval;

#[async_trait::async_trait]
impl AsyncCallable for Eval {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let (template, values) = match args {
            [template, Value::Object(values)] => (template, values),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        };

        let mut context = ctx.child();
        insert_values(&mut context, values.clone());
        let template = template.try_into()?;
        let ret = json_e::render_with_context(&template, &context).await?;
        Ok(ret.into())
    }
}

pub(super) struct EvalPlugin;

impl Plugin for EvalPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        context.insert(
            "eval",
            Value::Function(Function::new("eval", Box::new(Eval))),
        );
    }
}
