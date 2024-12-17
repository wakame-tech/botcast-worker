use crate::plugins::{default_plugins, Plugin};
use anyhow::Result;
use json_e::{builtins::builtins, value::Value, Context};
use std::collections::BTreeMap;

pub(crate) fn insert_values(context: &mut Context<'_>, values: BTreeMap<String, Value>) {
    for (k, v) in values {
        context.insert(k, v);
    }
}

pub struct ScriptRuntime<'a> {
    context: Context<'a>,
}

impl Default for ScriptRuntime<'_> {
    fn default() -> Self {
        let mut context = Context::new();
        builtins(&mut context);
        Self::new(default_plugins())
    }
}

impl ScriptRuntime<'_> {
    pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let mut context = Context::new();
        builtins(&mut context);
        for plugin in plugins {
            plugin.register_functions(&mut context);
        }
        Self { context }
    }

    pub fn install_plugin(&mut self, plugin: impl Plugin) {
        plugin.register_functions(&mut self.context);
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(
        &mut self,
        template: &serde_json::Value,
        values: BTreeMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        insert_values(
            &mut self.context,
            values.into_iter().map(|(k, v)| (k, v.into())).collect(),
        );
        json_e::render_with_context(template, &self.context).await
    }
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
