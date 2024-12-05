use anyhow::Result;
use api::client::ApiClient;

#[derive(Debug, clap::Parser)]
pub(crate) struct ListArgs {}

pub(crate) async fn cmd_list(client: ApiClient, _args: ListArgs) -> Result<()> {
    let scripts = client.scripts().await?;
    for script in scripts {
        println!("{}", serde_json::to_string_pretty(&script)?);
    }
    Ok(())
}
