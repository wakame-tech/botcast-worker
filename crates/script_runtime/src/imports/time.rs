use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};

#[derive(Clone)]
pub(crate) struct Today;

#[async_trait::async_trait]
impl AsyncCallable for Today {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let format = match args {
            [Value::String(format)] => Ok(format),
            _ => Err(anyhow::anyhow!("today only supports a string".to_string())),
        }?;
        let today = chrono::Local::now().format(&format).to_string();
        Ok(Value::String(today))
    }
}

#[cfg(test)]
mod tests {
    use crate::{imports::create_context, provider::DefaultProvider};
    use json_e::Context;

    #[tokio::test]
    async fn test_call_today() {
        let mut context = Context::new();
        create_context(DefaultProvider, &mut context);
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
