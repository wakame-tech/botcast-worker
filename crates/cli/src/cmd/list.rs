use crate::{credential::Credential, project::Project};
use anyhow::Result;
use api::client::ApiClient;

#[derive(Debug, clap::Parser)]
pub(crate) struct ListArgs {}

pub(crate) async fn cmd_list(project: Project, _args: ListArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);
    let scripts = client.scripts().await?;
    for script in scripts {
        println!("{}", serde_json::to_string_pretty(&script)?);
    }
    Ok(())
}
