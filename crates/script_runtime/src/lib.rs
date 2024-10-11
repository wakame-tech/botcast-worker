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
}
