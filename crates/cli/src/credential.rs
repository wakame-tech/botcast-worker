use anyhow::Result;
use std::{fs::OpenOptions, path::PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Credential {
    pub(crate) endpoint: String,
    pub(crate) token: String,
}

impl Credential {
    pub(crate) fn new(endpoint: String, token: String) -> Self {
        Self { endpoint, token }
    }

    pub(crate) fn save(&self, path: &PathBuf) -> Result<()> {
        let mut f = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
        serde_json::to_writer(&mut f, &self)?;
        Ok(())
    }

    pub(crate) fn load(path: &PathBuf) -> Result<Credential> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "No credential found, please execute `cli login` first"
            ));
        }
        let f = OpenOptions::new().read(true).open(path)?;
        let credential: Credential = serde_json::from_reader(f)?;
        Ok(credential)
    }
}
