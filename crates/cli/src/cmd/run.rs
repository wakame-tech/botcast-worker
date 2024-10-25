use anyhow::Result;
use std::{fs::File, path::PathBuf};

#[derive(Debug, clap::Parser)]
pub(crate) struct RunArgs {
    path: PathBuf,
    #[clap(long, default_value = "{}")]
    context: String,
}

pub(crate) async fn cmd_run(args: RunArgs) -> Result<()> {
    let template: serde_json::Value = serde_json::from_reader(File::open(&args.path)?)?;
    let context = serde_json::from_str(&args.context)?;
    let result = script_runtime::runtime::run(&template, context).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
