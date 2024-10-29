use crate::{
    api::{client::ApiClient, dto::UpdateScript},
    credential::Credential,
    project::Project,
};
use anyhow::Result;
use repos::entity::Script;
use std::fs::File;

#[derive(Debug, clap::Parser)]
pub(crate) struct PushArgs;

pub(crate) async fn cmd_push(project: Project, _args: PushArgs) -> Result<()> {
    let credential = Credential::load(&project.credential_path())?;
    let client = ApiClient::new(&credential.api_endpoint, &credential.token);
    for script_path in project.scripts_dir().read_dir()? {
        let entry = script_path?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_owned();
        let path = project.script_path(&file_name.to_string_lossy());
        let script: Script = serde_json::from_reader(File::open(&path)?)?;
        let input = UpdateScript::new(script.id, script.title, script.template);
        client.update_script(input).await?;

        println!("pushed {}", path.display());
    }
    Ok(())
}
