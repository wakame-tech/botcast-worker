mod api;
mod cmd;
mod credential;
mod project;
mod provider;

use anyhow::Result;
use clap::Parser;
use cmd::{Args, Cmd};
use project::Project;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::try_parse()?;
    let pwd = args.project.unwrap_or(std::env::current_dir()?);
    let project = Project::new(pwd);

    if let Cmd::Login(args) = args.cmd {
        return cmd::login::cmd_login(project, args);
    }

    match args.cmd {
        Cmd::New(args) => cmd::new::cmd_new(args)?,
        Cmd::List(args) => cmd::list::cmd_list(project, args)?,
        Cmd::Pull(args) => cmd::pull::cmd_pull(project, args)?,
        Cmd::Push(args) => cmd::push::cmd_push(project, args)?,
        Cmd::Add(args) => cmd::add::cmd_add(project, args)?,
        Cmd::Run(args) => cmd::run::cmd_run(project, args).await?,
        Cmd::Login(_) => (),
    };
    Ok(())
}
