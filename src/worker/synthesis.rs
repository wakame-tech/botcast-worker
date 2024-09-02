use super::{voicevox_client::VoiceVoxSpeaker, Args, RunTask};
use crate::{api::ctx::Ctx, repo::EpisodeRepo};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug)]
struct WorkDir(PathBuf, bool /* keep */);

impl WorkDir {
    fn new(task_id: &Uuid, keep: bool) -> anyhow::Result<Self> {
        let task_id = task_id.hyphenated().to_string();
        let work_dir = PathBuf::from("temp").join(&task_id);
        if !work_dir.exists() {
            std::fs::create_dir_all(&work_dir)?;
        }
        Ok(Self(PathBuf::from("temp").join(&task_id), keep))
    }

    fn dir(&self) -> &PathBuf {
        &self.0
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        if self.0.exists() && !self.1 {
            fs::remove_dir_all(&self.0).unwrap_or_else(|e| {
                log::error!("Failed to remove file: {}\n{}", self.0.display(), e);
            })
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Synthesis {
    pub(crate) episode_id: Uuid,
}

impl RunTask for Synthesis {
    async fn run(&self, task_id: Uuid, ctx: &Ctx) -> anyhow::Result<Option<Args>> {
        let repo = EpisodeRepo::new(ctx.pool.clone());
        let Some(mut episode) = repo.find_by_id(&self.episode_id).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };
        let text = episode
            .content
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

        let sentences = text.split('。').collect::<Vec<_>>();
        if sentences.is_empty() {
            anyhow::bail!("Sentences is empty");
        }

        let work_dir = WorkDir::new(&task_id, false)?;
        let speaker = VoiceVoxSpeaker::ZundaNormal;

        let mut paths = vec![];
        for (i, sentence) in sentences.iter().enumerate() {
            log::info!("[{}] {}", i, sentence);
            let sentence_wav_path = work_dir.dir().join(format!("{}.wav", i));
            paths.push(sentence_wav_path.clone());
            let query = match ctx.voicevox.query(sentence, &speaker).await {
                Ok(query) => query,
                Err(e) => {
                    log::error!("Failed to query: {}", e);
                    continue;
                }
            };
            match ctx
                .voicevox
                .synthesis(query, &speaker, &sentence_wav_path)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to synthesis: {}", e);
                    continue;
                }
            };
        }

        let episode_wav_path = concat_wavs(&work_dir, &paths).await?;
        let mut file = File::open(&episode_wav_path)?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)?;
        ctx.r2_client.upload_wav(&episode.id, &audio).await?;
        episode.audio_url = Some(ctx.r2_client.get_url(&episode.id));
        repo.update(&episode).await?;
        Ok(None)
    }
}

async fn concat_wavs(work_dir: &WorkDir, paths: &[PathBuf]) -> anyhow::Result<PathBuf> {
    let inputs_path = work_dir.dir().join("inputs.txt");
    let text = paths
        .iter()
        .map(|p| format!("file '{}'", p.file_name().unwrap().to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(&inputs_path)?;
    f.write_all(text.as_bytes())?;

    let episode_wav_path = work_dir.dir().join("episode.wav");
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        inputs_path.display().to_string().as_str(),
        episode_wav_path.display().to_string().as_str(),
    ]);
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!("Failed to concat wavs: {}", String::from_utf8(res.stderr)?);
    }
    Ok(episode_wav_path)
}

#[cfg(test)]
mod tests {
    use crate::worker::voicevox_client::{VoiceVox, VoiceVoxSpeaker};
    use std::path::PathBuf;
    use tokio::fs;

    #[tokio::test]
    async fn test_synthesis() -> anyhow::Result<()> {
        let voice_vox = VoiceVox::default();
        let speaker = VoiceVoxSpeaker::ZundaNormal;
        let query = voice_vox.query("こんにちは", &speaker).await?;
        let out = PathBuf::from("test.wav");
        voice_vox.synthesis(query, &speaker, &out).await?;
        fs::remove_file(out).await?;
        Ok(())
    }
}
