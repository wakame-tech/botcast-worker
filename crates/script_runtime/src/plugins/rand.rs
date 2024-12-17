use super::{as_array, as_u64, evaluate_args, Plugin};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use rand::{seq::SliceRandom, Rng};
use tracing::instrument;

#[derive(Clone)]
struct Rand;

#[async_trait::async_trait]
impl AsyncCallable for Rand {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let min = as_u64(&args[0])?;
        let max = as_u64(&args[1])?;
        let mut rng = rand::thread_rng();
        let ret = rng.gen_range(min..max);
        Ok(Value::Number(ret as f64))
    }
}

#[derive(Clone)]
struct Choice;

#[async_trait::async_trait]
impl AsyncCallable for Choice {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let arr = as_array(&args[0])?;
        let mut rng = rand::thread_rng();
        let ret = arr.choose(&mut rng).unwrap();
        Ok(ret.into())
    }
}

pub(crate) struct RandPlugin;

impl Plugin for RandPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        for (name, f) in [
            ("rand", Box::new(Rand) as Box<dyn AsyncCallable>),
            ("choice", Box::new(Choice)),
        ] {
            context.insert(name, Value::Function(Function::new(name, f)));
        }
    }
}
