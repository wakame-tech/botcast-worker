use crate::project::Project;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub(crate) struct NewArgs {
    dir: PathBuf,
}

pub(crate) fn cmd_new(args: NewArgs) -> Result<()> {
    let project = Project::new(args.dir);
    project.instantiate()
}
