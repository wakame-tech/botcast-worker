use add::AddArgs;
use login::LoginArgs;
use new::NewArgs;

pub(crate) mod add;
pub(crate) mod login;
pub(crate) mod new;

#[derive(Debug, clap::Parser)]
pub(crate) enum Args {
    Login(LoginArgs),
    New(NewArgs),
    Add(AddArgs),
}
