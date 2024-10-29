use add::AddArgs;
use list::ListArgs;
use login::LoginArgs;
use new::NewArgs;
use pull::PullArgs;
use push::PushArgs;
use run::RunArgs;
use std::path::PathBuf;

pub(crate) mod add;
pub(crate) mod list;
pub(crate) mod login;
pub(crate) mod new;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod run;

#[derive(Debug, clap::Parser)]
pub(crate) struct Args {
    #[clap(short, long)]
    pub(crate) project: Option<PathBuf>,
    #[clap(subcommand)]
    pub(crate) cmd: Cmd,
}

#[derive(Debug, clap::Parser)]
pub(crate) enum Cmd {
    Login(LoginArgs),
    New(NewArgs),
    List(ListArgs),
    Pull(PullArgs),
    Push(PushArgs),
    Add(AddArgs),
    Run(RunArgs),
}
