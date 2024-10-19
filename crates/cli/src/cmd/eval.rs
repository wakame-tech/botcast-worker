use crate::{credential::Credential, project::Project};
use anyhow::Result;
use std::fs::File;

#[derive(Debug, clap::Parser)]
pub(crate) struct EvalArgs {
    id: String,
}

pub(crate) fn cmd_eval(project: Project, args: EvalArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = reqwest::blocking::Client::new();

    let script_path = project.script_path(&args.id);
    let template: serde_json::Value = serde_json::from_reader(File::open(&script_path)?)?;

    let resp = client
        .post(format!("{}/evalScript", credential.worker_endpoint))
        .json(&template)
        .send()?;
    if resp.status() != 200 {
        anyhow::bail!("failed to eval script: {}", resp.text()?);
    }
    let result: serde_json::Value = resp.json()?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
