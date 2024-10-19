use crate::{
    api_client::{ApiClient, NewScript},
    credential::Credential,
    project::Project,
};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct AddArgs {
    title: String,
}

pub(crate) fn cmd_add(project: Project, args: AddArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::from_credential(&credential);

    let input = NewScript::new(args.title);
    let script = client.new_script(input)?;
    let path = project.instantiate_script(&script)?;
    println!("created {}", path.display());
    Ok(())
}
