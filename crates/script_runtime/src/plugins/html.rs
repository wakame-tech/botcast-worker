use super::{as_string, evaluate_args, Plugin};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use readable_text::ReadableText;
use tracing::instrument;

#[derive(Clone)]
struct Text;

#[async_trait::async_trait]
impl AsyncCallable for Text {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let html = as_string(&evaluated[0])?;
        let md = ReadableText::extract(&html)?;
        Ok(Value::String(md))
    }
}

#[derive(Default)]
pub(crate) struct HtmlPlugin;

impl Plugin for HtmlPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        {
            let (name, f) = ("text", Box::new(Text));
            context.insert(name, Value::Function(Function::new(name, f)));
        }
    }
}
