use crate::{project::Project, provider::LocalFileScriptRepo};
use anyhow::Result;
use repos::provider::{
    DefaultProvider, ProvideCommentRepo, ProvideEpisodeRepo, ProvidePodcastRepo,
};
use script_runtime::{imports::urn::UrnGet, runtime::ScriptRuntime};
use std::{fs::File, path::PathBuf, sync::Arc};

#[derive(Debug, clap::Parser)]
pub(crate) struct RunArgs {
    path: PathBuf,
    #[clap(long, default_value = "{}")]
    context: String,
}

pub(crate) async fn cmd_run(project: Project, args: RunArgs) -> Result<()> {
    let template: serde_json::Value = serde_json::from_reader(File::open(&args.path)?)?;
    let context = serde_json::from_str(&args.context)?;
    let mut runtime = ScriptRuntime::new();
    runtime.register_function(
        "get",
        Box::new(UrnGet::new(
            DefaultProvider.podcast_repo(),
            DefaultProvider.episode_repo(),
            DefaultProvider.comment_repo(),
            Arc::new(LocalFileScriptRepo::new(project.scripts_dir())),
        )),
    );
    let result = runtime.run(&template, context).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}