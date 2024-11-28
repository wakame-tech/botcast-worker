use crate::{credential::Credential, project::Project};
use anyhow::Result;
use api::{client::ApiClient, script::NewScript};

#[derive(Debug, clap::Parser)]
pub(crate) struct AddArgs {
    title: String,
}

pub(crate) async fn cmd_add(project: Project, args: AddArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);

    let input = NewScript::new(args.title);
    let script = client.new_script(input).await?;
    let path = project.instantiate_script(&script)?;
    println!("created {}", path.display());
    Ok(())
}
