mod fetch;
mod jq;
mod llm;
mod time;
pub mod urn;

use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};

pub(crate) fn insert_custom_functions(context: &mut Context) {
    let functions = [
        ("today", Box::new(time::Today) as Box<dyn AsyncCallable>),
        ("eval", Box::new(urn::Eval)),
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
