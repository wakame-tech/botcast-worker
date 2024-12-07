mod audio_downloader;
pub mod ffmpeg;
pub mod generate_audio;
pub mod voicevox;
pub mod workdir;

use async_trait::async_trait;
use generate_audio::SectionSegment;
use workdir::WorkDir;

#[async_trait]
pub trait AudioGenerator: Send + Sync {
    async fn generate(&self, workdir: &WorkDir, segment: SectionSegment)
        -> anyhow::Result<Vec<u8>>;
}
