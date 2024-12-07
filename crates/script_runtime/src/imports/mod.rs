pub mod api;
pub mod eval;
mod fetch;
mod jq;
pub mod llm;
mod time;

use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};

fn as_string(value: &serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Err(anyhow::anyhow!("expected a string")),
    }
}

pub(crate) fn insert_custom_functions(context: &mut Context) {
    let functions = [
        ("today", Box::new(time::Today) as Box<dyn AsyncCallable>),
        ("eval", Box::new(eval::Eval)),
        ("fetch", Box::new(fetch::Fetch)),
        ("fetch_json", Box::new(fetch::FetchJson)),
        ("text", Box::new(fetch::Text)),
        ("jq", Box::new(jq::Jq)),
        ("hq", Box::new(jq::Hq)),
        ("replace", Box::new(jq::Replace)),
    ];
    for (name, f) in functions.into_iter() {
        context.insert(name.to_string(), Value::Function(Function::new(name, f)));
    }
}
