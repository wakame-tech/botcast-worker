use crate::{api_client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct PullArgs {}

pub(crate) fn cmd_pull(project: Project, _args: PullArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::from_credential(&credential);

    let scripts = client.scripts()?;
    for script in scripts {
        let path = project.instantiate_script(&script)?;
        println!("pulled {}", path.display());
    }
    Ok(())
}
