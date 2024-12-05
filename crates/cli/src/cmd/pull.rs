use crate::project::Project;
use anyhow::Result;
use api::client::ApiClient;

#[derive(Debug, clap::Parser)]
pub(crate) struct PullArgs;

pub(crate) async fn cmd_pull(client: ApiClient, project: Project, _args: PullArgs) -> Result<()> {
    let scripts = client.scripts().await?;
    for script in scripts {
        match project.instantiate_script(&script) {
            Ok(path) => {
                println!("pulled {}", path.display());
            }
            Err(e) => {
                eprintln!("skipped: {}", e);
            }
        }
    }
    Ok(())
}
