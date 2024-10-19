use crate::api_client::Script;
use anyhow::Result;
use std::{io::Write, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Project {
    root: PathBuf,
}

impl Project {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn credential_path(&self) -> PathBuf {
        self.root.join(".credential.json")
    }

    pub(crate) fn script_path(id: String) -> PathBuf {
        PathBuf::from("scripts").join(format!("{}.json", id))
    }

    pub(crate) fn instantiate_script(&self, script: &Script) -> Result<PathBuf> {
        let path = self.root.join(Self::script_path(script.id.clone()));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)?;
        serde_json::to_writer_pretty(&mut f, &script.template)?;
        Ok(path)
    }

    pub(crate) fn instantiate(&self) -> Result<()> {
        anyhow::ensure!(!self.root.exists(), "{} exists", self.root.display());
        std::fs::create_dir_all(&self.root)?;
        let templates = [
            (PathBuf::from(".gitignore"), Some(r#".credential.json"#)),
            (PathBuf::from("scripts"), None),
        ];

        for (path, content) in templates.iter() {
            let path = self.root.join(path);
            match content {
                Some(content) => {
                    let mut f = std::fs::File::create(&path)?;
                    f.write_all(content.as_bytes())?;
                    println!("created {}", path.display());
                }
                None => {
                    std::fs::create_dir_all(&path)?;
                    println!("created {}/", path.display());
                }
            }
        }
        Ok(())
    }
}
