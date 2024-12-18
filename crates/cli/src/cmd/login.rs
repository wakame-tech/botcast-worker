use crate::{credential::Credential, project::Project};
use anyhow::Result;
use api::client::ApiClient;

#[derive(Debug, clap::Parser)]
pub(crate) struct LoginArgs {
    #[clap(long)]
    email: String,
    #[clap(long)]
    password: String,
}

pub(crate) async fn cmd_login(project: Project, args: LoginArgs) -> Result<()> {
    let mut credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, "");
    let token = client.sign_in(&args.email, &args.password).await?;
    credential.token = token;

    let path = project.credential_path();
    credential.save(&path)?;
    println!("credential updated {}", path.display());
    Ok(())
}
