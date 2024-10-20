mod fetch;
mod jq;
mod llm;
mod time;
mod urn;

use crate::provider::DefaultProvider;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use std::collections::BTreeMap;

fn display_fn_io(name: &str, args: &[Value], ret: &Result<serde_json::Value>) -> Result<String> {
    Ok(format!(
        "{}(\n{}\n) = {}",
        name,
        args.iter()
            .map(serde_json::Value::try_from)
            .collect::<Result<Vec<_>>>()?
            .iter()
            .map(|v| serde_json::to_string_pretty(v).unwrap())
            .collect::<Vec<_>>()
            .join(",\n"),
        match ret {
            Ok(v) => format!("Ok({})", serde_json::to_string_pretty(&v).unwrap()),
            Err(e) => format!("Err({})", e),
        },
    ))
}

pub(crate) fn insert_custom_functions(provider: DefaultProvider, context: &mut Context) {
    let functions = [
        ("today", Box::new(time::Today) as Box<dyn AsyncCallable>),
        ("eval", Box::new(urn::Eval)),
        ("get", Box::new(urn::UrnGet { provider })),
        ("fetch", Box::new(fetch::Fetch)),
        ("fetch_json", Box::new(fetch::FetchJson)),
        ("text", Box::new(fetch::Text)),
        ("llm", Box::new(llm::Llm)),
        ("jq", Box::new(jq::Jq)),
    ];
    for (name, f) in functions.into_iter() {
        context.insert(name.to_string(), Value::Function(Function::new(name, f)));
    }
}

pub(crate) fn insert_values<'a>(context: &mut Context, values: BTreeMap<String, Value>) {
    for (k, v) in values {
        context.insert(k, v);
    }
}
