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

    pub(crate) fn instantiate(&self) -> Result<()> {
        anyhow::ensure!(!self.root.exists(), "{} exists", self.root.display());
        std::fs::create_dir_all(&self.root)?;
        let templates = [(PathBuf::from(".gitignore"), r#".credential.json"#)];

        for (path, content) in templates.iter() {
            let path = self.root.join(path);
            let mut f = std::fs::File::create(&path)?;
            f.write_all(content.as_bytes())?;
            println!("created {:?}", path);
        }
        Ok(())
    }
}
