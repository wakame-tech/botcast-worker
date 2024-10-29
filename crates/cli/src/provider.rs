use anyhow::Result;
use async_trait::async_trait;
use repos::{
    entity::{Script, ScriptId},
    repo::ScriptRepo,
};
use std::{fs::File, path::PathBuf};

pub(crate) struct LocalFileScriptRepo {
    scripts_dir: PathBuf,
}

impl LocalFileScriptRepo {
    pub(crate) fn new(scripts_dir: PathBuf) -> LocalFileScriptRepo {
        LocalFileScriptRepo { scripts_dir }
    }
}

#[async_trait]
impl ScriptRepo for LocalFileScriptRepo {
    async fn find_by_id(&self, id: &ScriptId) -> Result<Script, repos::error::Error> {
        let path = self
            .scripts_dir
            .join(format!("{}.json", id.0.as_hyphenated()));
        dbg!(&path);
        let mut file = File::open(&path)
            .map_err(|_| repos::error::Error::NotFound("script".to_string(), id.0.to_string()))?;
        let template: serde_json::Value = serde_json::from_reader(&mut file)
            .map_err(|_| repos::error::Error::NotFound("script".to_string(), id.0.to_string()))?;
        Ok(Script {
            id: id.0,
            user_id: id.0,
            title: format!("{}", id.0.as_hyphenated()),
            template,
            result: None,
        })
    }

    async fn update(&self, _script: &Script) -> Result<(), repos::error::Error> {
        unreachable!()
    }
}
