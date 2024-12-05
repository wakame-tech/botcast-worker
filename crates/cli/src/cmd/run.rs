use crate::project::Project;
use anyhow::Result;
use api::client::ApiClient;
use script_runtime::{
    imports::{api::register_api_functions, llm::register_llm_functions},
    runtime::ScriptRuntime,
};
use std::{fs::File, path::PathBuf, sync::Arc};

#[derive(Debug, clap::Parser)]
pub(crate) struct RunArgs {
    path: PathBuf,
    #[clap(long, default_value = "{}")]
    context: String,
}

pub(crate) async fn cmd_run(client: ApiClient, _project: Project, args: RunArgs) -> Result<()> {
    let template: serde_json::Value = serde_json::from_reader(File::open(&args.path)?)?;
    let context = serde_json::from_str(&args.context)?;
    let mut runtime = ScriptRuntime::default();
    register_api_functions(&mut runtime, Arc::new(client));
    register_llm_functions(&mut runtime);
    let result = runtime.run(&template, context).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
