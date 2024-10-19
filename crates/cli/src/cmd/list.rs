use crate::{api::client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct ListArgs {}

pub(crate) fn cmd_list(project: Project, _args: ListArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);
    let scripts = client.scripts()?;
    for script in scripts {
        println!("{}", serde_json::to_string_pretty(&script)?);
    }
    Ok(())
}
