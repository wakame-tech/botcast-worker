use add::AddArgs;
use list::ListArgs;
use login::LoginArgs;
use new::NewArgs;
use pull::PullArgs;
use push::PushArgs;
use run::RunArgs;

pub(crate) mod add;
pub(crate) mod list;
pub(crate) mod login;
pub(crate) mod new;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod run;

#[derive(Debug, clap::Parser)]
pub(crate) enum Args {
    Login(LoginArgs),
    New(NewArgs),
    List(ListArgs),
    Pull(PullArgs),
    Push(PushArgs),
    Add(AddArgs),
    Run(RunArgs),
}
