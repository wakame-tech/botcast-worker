use crate::{
    api_client::{ApiClient, UpdateScript},
    credential::Credential,
    project::Project,
};
use anyhow::Result;
use std::fs::File;

#[derive(Debug, clap::Parser)]
pub(crate) struct PushArgs {
    id: String,
}

pub(crate) fn cmd_push(project: Project, args: PushArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::from_credential(&credential);

    let script = client.script(&args.id)?;
    let path = project.script_path(&args.id);
    let template: serde_json::Value = serde_json::from_reader(File::open(&path)?)?;
    let input = UpdateScript::new(script.id, script.title, template);
    client.update_script(input)?;

    println!("pushed {}", path.display());
    Ok(())
}
