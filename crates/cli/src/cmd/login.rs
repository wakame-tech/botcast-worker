use crate::{api_client::ApiClient, credential::Credential, project::Project};
use anyhow::Result;

#[derive(Debug, clap::Parser)]
pub(crate) struct LoginArgs {
    endpoint: String,
    #[clap(long)]
    email: String,
    #[clap(long)]
    password: String,
}

pub(crate) fn cmd_login(project: Project, args: LoginArgs) -> Result<()> {
    let client = ApiClient::new(&args.endpoint);
    let token = client.sign_in(&args.email, &args.password)?;
    let credential = Credential::new(args.endpoint, token);

    let path = project.credential_path();
    credential.save(&path)?;
    println!("credential saved to {}", path.display());
    Ok(())
}
