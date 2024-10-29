use crate::{api::client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct PullArgs;

pub(crate) async fn cmd_pull(project: Project, _args: PullArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);

    let scripts = client.scripts().await?;
    for script in scripts {
        let path = project.instantiate_script(&script)?;
        println!("pulled {}", path.display());
    }
    Ok(())
}
