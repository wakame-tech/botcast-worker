use std::{fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub struct WorkDir(PathBuf, bool /* keep */);

impl WorkDir {
    pub fn new(task_id: &Uuid, keep: bool) -> anyhow::Result<Self> {
        let task_id = task_id.hyphenated().to_string();
        let work_dir = PathBuf::from("temp").join(&task_id);
        if !work_dir.exists() {
            std::fs::create_dir_all(&work_dir)?;
        }
        Ok(Self(PathBuf::from("temp").join(&task_id), keep))
    }

    pub fn dir(&self) -> &PathBuf {
        &self.0
    }

    pub fn is_keep_dir(&self) -> bool {
        self.1
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        if self.0.exists() && !self.1 {
            fs::remove_dir_all(&self.0).unwrap_or_else(|e| {
                tracing::error!("Failed to remove file: {}\n{}", self.0.display(), e);
            })
        }
    }
}
