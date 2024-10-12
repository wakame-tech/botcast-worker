use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{value::Value, Context};

pub fn today<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        match args {
            [Value::String(format)] => {
                let today = chrono::Local::now().format(&format).to_string();
                Ok(Value::String(today))
            }
            _ => Err(anyhow::anyhow!("today only supports a string".to_string())),
        }
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use crate::imports::define_imports;

    #[tokio::test]
    async fn test_call_today() {
        let mut context = json_e::Context::new();
        define_imports(&mut context);
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
