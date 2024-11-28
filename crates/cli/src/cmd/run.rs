use crate::{credential::Credential, project::Project};
use anyhow::Result;
use api::client::ApiClient;
use script_runtime::{
    imports::{
        llm::{create_thread, delete_thread, register_llm_functions},
        repo::register_repo_functions,
    },
    runtime::ScriptRuntime,
};
use std::{fs::File, path::PathBuf, sync::Arc};

#[derive(Debug, clap::Parser)]
pub(crate) struct RunArgs {
    path: PathBuf,
    #[clap(long, default_value = "{}")]
    context: String,
}

pub(crate) async fn cmd_run(project: Project, args: RunArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = Arc::new(ApiClient::new(&credential.api_endpoint, &credential.token));

    let template: serde_json::Value = serde_json::from_reader(File::open(&args.path)?)?;
    let context = serde_json::from_str(&args.context)?;
    let mut runtime = ScriptRuntime::default();
    register_repo_functions(&mut runtime, client);
    let open_ai_api_key = std::env::var("OPENAI_API_KEY")?;
    let thread_id = create_thread(open_ai_api_key.clone()).await?;
    register_llm_functions(&mut runtime, open_ai_api_key.clone(), thread_id.clone());
    let result = runtime.run(&template, context).await?;
    delete_thread(open_ai_api_key, thread_id).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
