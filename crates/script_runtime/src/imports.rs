use crate::Urn;
use anyhow::Result;
use json_e::{
    value::{Function, Value},
    Context,
};

pub(crate) struct Imports;

impl Imports {
    pub(crate) fn today(&self, format: String) -> String {
        chrono::Local::now().format(&format).to_string()
    }

    pub(crate) fn get(&self, urn: Urn) -> String {
        // TODO
        urn.0
    }

    pub(crate) fn fetch(&self, url: String) -> String {
        // TODO
        url
    }

    pub(crate) fn llm(&self, _prompt: String) -> String {
        // TODO
        "llm".to_string()
    }
}

fn today(_: &Context<'_>, args: &[Value]) -> Result<Value> {
    let imports = Imports;
    match args {
        [Value::String(format)] => Ok(Value::String(imports.today(format.clone()))),
        _ => Err(anyhow::anyhow!("today only supports a string".to_string())),
    }
}

fn get(_: &Context<'_>, args: &[Value]) -> Result<Value> {
    let imports = Imports;
    match args {
        [Value::String(urn)] => Ok(Value::String(imports.get(Urn(urn.clone())))),
        _ => Err(anyhow::anyhow!("get only supports a string".to_string())),
    }
}

fn fetch(_: &Context<'_>, args: &[Value]) -> Result<Value> {
    let imports = Imports;
    match args {
        [Value::String(url)] => Ok(Value::String(imports.fetch(url.clone()))),
        _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
    }
}

fn llm(_: &Context<'_>, args: &[Value]) -> Result<Value> {
    let imports = Imports;
    match args {
        [Value::String(prompt)] => Ok(Value::String(imports.llm(prompt.clone()))),
        _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
    }
}

pub(crate) fn define_imports<'a>(context: &mut Context<'a>) {
    context.insert("today", Value::Function(Function::new("today", today)));
    context.insert("get", Value::Function(Function::new("get", get)));
    context.insert("fetch", Value::Function(Function::new("fetch", fetch)));
    context.insert("llm", Value::Function(Function::new("llm", llm)));
}
