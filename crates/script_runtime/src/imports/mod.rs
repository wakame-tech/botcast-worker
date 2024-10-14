mod fetch;
mod llm;
mod time;
mod urn;

use json_e::{
    value::{Function, Value},
    Context,
};

pub fn define_imports<'a>(context: &mut Context<'a>) {
    context.insert(
        "today",
        Value::Function(Function::new("today", time::today)),
    );
    context.insert("get", Value::Function(Function::new("get", urn::get)));
    context.insert(
        "fetch",
        Value::Function(Function::new("fetch", fetch::fetch)),
    );
    context.insert("text", Value::Function(Function::new("text", fetch::text)));
    context.insert("llm", Value::Function(Function::new("llm", llm::llm)));
}
