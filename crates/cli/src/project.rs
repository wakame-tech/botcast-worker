use crate::credential::Credential;
use anyhow::Result;
use api::script::Script;
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

    pub(crate) fn scripts_dir(&self) -> PathBuf {
        self.root.join("scripts")
    }

    pub(crate) fn script_path(&self, id: &str) -> PathBuf {
        self.scripts_dir().join(format!("{}.json", id))
    }

    pub(crate) fn instantiate_script(&self, script: &Script) -> Result<PathBuf> {
        let path = self.script_path(&script.id);
        if path.exists() {
            anyhow::bail!("{} exists", path.display());
        }
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;
        serde_json::to_writer_pretty(&mut f, &script)?;
        Ok(path)
    }

    pub(crate) fn instantiate(&self) -> Result<()> {
        anyhow::ensure!(!self.root.exists(), "{} exists", self.root.display());
        std::fs::create_dir_all(&self.root)?;

        let credential = serde_json::to_string_pretty(&Credential::default())?;
        let templates = [
            (PathBuf::from(".gitignore"), Some(r#".credential.json"#)),
            (PathBuf::from(".credential.json"), Some(&credential)),
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
