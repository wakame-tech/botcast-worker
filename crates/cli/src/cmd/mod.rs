use add::AddArgs;
use list::ListArgs;
use login::LoginArgs;
use new::NewArgs;
use pull::PullArgs;

pub(crate) mod add;
pub(crate) mod list;
pub(crate) mod login;
pub(crate) mod new;
pub(crate) mod pull;

#[derive(Debug, clap::Parser)]
pub(crate) enum Args {
    Login(LoginArgs),
    New(NewArgs),
    List(ListArgs),
    Pull(PullArgs),
    Add(AddArgs),
}
