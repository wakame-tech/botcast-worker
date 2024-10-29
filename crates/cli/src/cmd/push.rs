use crate::{
    api::{client::ApiClient, dto::UpdateScript},
    credential::Credential,
    project::Project,
};
use anyhow::Result;
use std::fs::File;

#[derive(Debug, clap::Parser)]
pub(crate) struct PushArgs {
    id: String,
}

pub(crate) async fn cmd_push(project: Project, args: PushArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);

    let script = client.script(&args.id).await?;
    let path = project.script_path(&args.id);
    let template: serde_json::Value = serde_json::from_reader(File::open(&path)?)?;
    let input = UpdateScript::new(script.id, script.title, template);
    client.update_script(input).await?;

    println!("pushed {}", path.display());
    Ok(())
}
