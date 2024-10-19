mod api_client;
mod cmd;
mod credential;
mod project;
mod trpc;

use anyhow::Result;
use clap::Parser;
use cmd::Args;
use credential::Credential;
use project::Project;

fn main() -> Result<()> {
    let args = Args::try_parse()?;
    let pwd = std::env::current_dir()?;
    let project = Project::new(pwd);

    if let Args::Login(args) = args {
        return cmd::login::cmd_login(project, args);
    }

    match args {
        Args::New(args) => cmd::new::cmd_new(args)?,
        Args::List(args) => cmd::list::cmd_list(project, args)?,
        Args::Pull(args) => cmd::pull::cmd_pull(project, args)?,
        Args::Push(args) => cmd::push::cmd_push(project, args)?,
        Args::Add(args) => cmd::add::cmd_add(project, args)?,
        Args::Login(_) => (),
    };
    Ok(())
}
