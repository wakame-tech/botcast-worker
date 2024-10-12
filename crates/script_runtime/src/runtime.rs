use crate::{imports::define_imports, Manuscript};
use anyhow::Result;
use json_e::Context;

pub fn run(template: &serde_json::Value) -> Result<Manuscript> {
    let mut context = Context::default();
    define_imports(&mut context);
    let result = json_e::render(template, &context)?;
    let manuscript: Manuscript = serde_json::from_value(result)?;
    Ok(manuscript)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use json_e::{
        value::{Function, Value},
        Context,
    };
    use serde_json::json;

    fn add(_: &Context<'_>, args: &[Value]) -> Result<Value> {
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            _ => Err(anyhow::anyhow!("add only supports numbers, got {:?}", args)),
        }
    }

    fn custom_context<'a>() -> Context<'a> {
        let mut context = Context::default();
        context.insert("today", Value::String("xx月yy日".to_string()));
        context.insert("add", Value::Function(Function::new("add", add)));
        context
    }

    #[test]
    fn test_custom_function() -> Result<()> {
        let template = json!("こんにちは、${today} 1+2=${add(1, 2)}");
        let context = custom_context();
        let result = json_e::render(&template, &context)?;
        assert_eq!(result, json!("こんにちは、xx月yy日 1+2=3"));
        Ok(())
    }

    #[test]
    fn test_nested_let() -> Result<()> {
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
        let result = json_e::render(&template, &context)?;
        assert_eq!(
            result,
            json!(serde_json::to_string(&json!({"b": "hoge", "c": "3"})).unwrap()),
        );
        Ok(())
    }
}
