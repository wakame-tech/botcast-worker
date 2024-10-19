use crate::{api_client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct ListArgs {}

pub(crate) fn cmd_list(project: Project, _args: ListArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::from_credential(&credential);
    let scripts = client.scripts()?;
    println!("{:?}", scripts);
    Ok(())
}
