pub mod ffmpeg;
pub mod generate_audio;
pub mod voicevox;
pub mod workdir;

mod audio_downloader;

use api::episode::Section;
use async_trait::async_trait;
use std::path::PathBuf;
use workdir::WorkDir;

#[async_trait]
pub trait AudioGenerator: Send + Sync {
    async fn generate(
        &self,
        i: &mut usize,
        workdir: &WorkDir,
        section: Section,
    ) -> anyhow::Result<Vec<(PathBuf, String)>>;
}
