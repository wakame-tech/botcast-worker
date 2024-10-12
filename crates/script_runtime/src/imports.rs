use crate::Urn;
use anyhow::Result;
use futures::{future::BoxFuture, FutureExt};
use json_e::{
    value::{Function, Value},
    Context,
};

pub(crate) struct Imports;

impl Imports {
    pub(crate) async fn today(&self, format: String) -> String {
        chrono::Local::now().format(&format).to_string()
    }

    pub(crate) async fn get(&self, urn: Urn) -> String {
        // TODO
        urn.0
    }

    pub(crate) async fn fetch(&self, url: String) -> String {
        // TODO
        url
    }

    pub(crate) async fn llm(&self, _prompt: String) -> String {
        // TODO
        "llm".to_string()
    }
}

fn today<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports;
        match args {
            [Value::String(format)] => Ok(Value::String(imports.today(format.clone()).await)),
            _ => Err(anyhow::anyhow!("today only supports a string".to_string())),
        }
    }
    .boxed()
}

fn get<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports;
        match args {
            [Value::String(urn)] => Ok(Value::String(imports.get(Urn(urn.clone())).await)),
            _ => Err(anyhow::anyhow!("get only supports a string".to_string())),
        }
    }
    .boxed()
}

fn fetch<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports;
        match args {
            [Value::String(url)] => Ok(Value::String(imports.fetch(url.clone()).await)),
            _ => Err(anyhow::anyhow!("fetch only supports a string".to_string())),
        }
    }
    .boxed()
}

fn llm<'a>(_: &Context<'_>, args: &'a [Value]) -> BoxFuture<'a, Result<Value>> {
    async move {
        let imports = Imports;
        match args {
            [Value::String(prompt)] => Ok(Value::String(imports.llm(prompt.clone()).await)),
            _ => Err(anyhow::anyhow!("llm only supports a string".to_string())),
        }
    }
    .boxed()
}

pub(crate) fn define_imports<'a>(context: &mut Context<'a>) {
    context.insert("today", Value::Function(Function::new("today", today)));
    context.insert("get", Value::Function(Function::new("get", get)));
    context.insert("fetch", Value::Function(Function::new("fetch", fetch)));
    context.insert("llm", Value::Function(Function::new("llm", llm)));
}
