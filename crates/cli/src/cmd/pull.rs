use crate::{api::client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct PullArgs {
    id: String,
}

pub(crate) fn cmd_pull(project: Project, args: PullArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);

    let script = client.script(&args.id)?;
    let path = project.instantiate_script(&script)?;
    println!("pulled {}", path.display());
    Ok(())
}