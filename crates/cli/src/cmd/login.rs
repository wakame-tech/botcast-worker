use crate::{api::client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct LoginArgs {
    #[clap(long)]
    email: String,
    #[clap(long)]
    password: String,
}

pub(crate) fn cmd_login(project: Project, args: LoginArgs) -> Result<()> {
    let mut credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, "");
    let token = client.sign_in(&args.email, &args.password)?;
    credential.token = token;

    let path = project.credential_path();
    credential.save(&path)?;
    println!("credential updated {}", path.display());
    Ok(())
}
