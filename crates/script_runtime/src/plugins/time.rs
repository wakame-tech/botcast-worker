use super::{as_string, evaluate_args, Plugin};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use tracing::instrument;

#[derive(Clone)]
struct Today;

#[async_trait::async_trait]
impl AsyncCallable for Today {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let format = as_string(&evaluated[0])?;
        let today = chrono::Local::now().format(&format).to_string();
        Ok(Value::String(today))
    }
}

pub(crate) struct TimePlugin;

impl Plugin for TimePlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        {
            let (name, f) = ("today", Box::new(Today) as Box<dyn AsyncCallable>);
            context.insert(name, Value::Function(Function::new(name, f)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use json_e::Context;

    #[tokio::test]
    async fn test_call_today() {
        let mut context = Context::new();
        TimePlugin.register_functions(&mut context);
        let result = json_e::render_with_context(
            &serde_json::json!({ "$eval": "today('%Y/%m/%d')" }),
            &context,
        )
        .await
        .unwrap();
        let expected = serde_json::json!(chrono::Local::now().format("%Y/%m/%d").to_string());
        assert_eq!(result, expected);
    }
}
