mod cmd;
mod credential;
mod project;

use anyhow::Result;
use clap::Parser;
use cmd::{Args, Cmd};
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

    match args.cmd {
        Cmd::New(args) => cmd::new::cmd_new(args)?,
        Cmd::List(args) => cmd::list::cmd_list(project, args).await?,
        Cmd::Pull(args) => cmd::pull::cmd_pull(project, args).await?,
        Cmd::Push(args) => cmd::push::cmd_push(project, args).await?,
        Cmd::Add(args) => cmd::add::cmd_add(project, args).await?,
        Cmd::Run(args) => cmd::run::cmd_run(project, args).await?,
        Cmd::Login(_) => (),
    };
    Ok(())
}
