use crate::{credential::Credential, project::Project};
use anyhow::Result;
use api::client::ApiClient;

#[derive(Debug, clap::Parser)]
pub(crate) struct PullArgs;

pub(crate) async fn cmd_pull(project: Project, _args: PullArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);
    let scripts = client.scripts().await?;
    for script in scripts {
        match project.instantiate_script(&script) {
            Ok(path) => {
                println!("pulled {}", path.display());
            }
            Err(e) => {
                eprintln!("skipped: {}", e);
            }
        }
    }
    Ok(())
}
