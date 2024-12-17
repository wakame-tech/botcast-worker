use anyhow::Result;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkDir {
    id: String,
    dir: PathBuf,
    keep: bool,
}

impl WorkDir {
    pub fn new(task_id: &Uuid, keep: bool) -> anyhow::Result<Self> {
        let task_id = task_id.hyphenated().to_string();
        let dir = if keep {
            PathBuf::from("temp").join("workdir")
        } else {
            PathBuf::from("temp").join(&task_id)
        };
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        fs::OpenOptions::new()
            .append(true)
            .create(true)
            
            .open(dir.join(format!("{}.log", task_id)))?;

        Ok(Self {
            id: task_id.clone(),
            dir,
            keep,
        })
    }

    pub fn open_log(&self) -> Result<File> {
        let log_path = self.dir.join(format!("{}.log", self.id));
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            
            .open(log_path)
            .map_err(Into::into)
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn is_keep_dir(&self) -> bool {
        self.keep
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        if self.dir.exists() && !self.keep {
            fs::remove_dir_all(&self.dir).unwrap_or_else(|e| {
                tracing::error!("Failed to remove file: {}\n{}", self.dir.display(), e);
            })
        }
    }
}
