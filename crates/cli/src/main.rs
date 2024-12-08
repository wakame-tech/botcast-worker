mod cmd;
mod credential;
mod project;

use anyhow::Result;
use api::client::ApiClient;
use clap::Parser;
use cmd::{Args, Cmd};
use credential::Credential;
use project::Project;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let args = Args::try_parse()?;
    let pwd = args.project.unwrap_or(std::env::current_dir()?);
    let project = Project::new(pwd);

    if let Cmd::Login(args) = args.cmd {
        return cmd::login::cmd_login(project, args).await;
    }

    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);

    match args.cmd {
        Cmd::New(args) => cmd::new::cmd_new(args)?,
        Cmd::List(args) => cmd::list::cmd_list(client, args).await?,
        Cmd::Pull(args) => cmd::pull::cmd_pull(client, project, args).await?,
        Cmd::Push(args) => cmd::push::cmd_push(client, project, args).await?,
        Cmd::Add(args) => cmd::add::cmd_add(client, project, args).await?,
        Cmd::Run(args) => cmd::run::cmd_run(client, project, args).await?,
        Cmd::Login(_) => (),
    };
    Ok(())
}
