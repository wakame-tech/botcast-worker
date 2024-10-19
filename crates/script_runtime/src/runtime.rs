use crate::imports::create_context;
use anyhow::Result;
use json_e::Context;

pub async fn run(template: &serde_json::Value) -> Result<serde_json::Value> {
    let mut context = Context::new();
    create_context(&mut context);
    json_e::render_with_context(template, &context).await
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use json_e::{
        value::{AsyncCallable, Function, Value},
        Context,
    };
    use serde_json::json;

    #[derive(Clone)]
    struct Add;

    #[async_trait::async_trait]
    impl AsyncCallable for Add {
        async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
            match (&args[0], &args[1]) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                _ => Err(anyhow::anyhow!("add only supports numbers, got {:?}", args)),
            }
        }
    }

    fn custom_context<'a>() -> Context<'a> {
        let mut context = Context::new();
        context.insert("today", Value::String("xx月yy日".to_string()));
        context.insert("add", Value::Function(Function::new("add", Box::new(Add))));
        context
    }

    #[tokio::test]
    async fn test_custom_function() -> Result<()> {
        let template = json!("こんにちは、${today} 1+2=${add(1, 2)}");
        let context = custom_context();
        let result = json_e::render_with_context(&template, &context).await?;
        assert_eq!(result, json!("こんにちは、xx月yy日 1+2=3"));
        Ok(())
    }

    #[tokio::test]
    async fn test_nested_let() -> Result<()> {
        let template = json!({
            "$let": {
                "a": { "$eval": "add(1, 2)" },
            },
            "in": {
                "$let": {
                    "params": {
                        "$json": {
                            "b": "hoge",
                            "c": "${a}"
                        },
                    },
                },
                "in": { "$eval": "params" },
            }
        });
        let context = custom_context();
        let result = json_e::render_with_context(&template, &context).await?;
        assert_eq!(
            result,
            json!(serde_json::to_string(&json!({"b": "hoge", "c": "3"})).unwrap()),
        );
        Ok(())
    }
}
