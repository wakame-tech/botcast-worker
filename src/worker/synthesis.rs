use super::{voicevox_client::VoiceVoxSpeaker, Args, RunTask};
use crate::{api::ctx::Ctx, repo::EpisodeRepo};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};
use tokio::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Synthesis {
    pub(crate) episode_id: Uuid,
}

impl RunTask for Synthesis {
    async fn run(&self, task_id: Uuid, ctx: &Ctx) -> anyhow::Result<Option<Args>> {
        let repo = EpisodeRepo::new(ctx.pool.clone());
        let Some(episode) = repo.find_by_id(&self.episode_id).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };
        let text = episode
            .content
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

        let sentences = text.split('。').collect::<Vec<_>>();
        let dir = PathBuf::from("temp");
        if !dir.exists() {
            std::fs::create_dir(&dir)?;
        }
        let text_path = dir.join("text.txt");

        let speaker = VoiceVoxSpeaker::ZundaNormal;
        let mut artifacts: Vec<PathBuf> = vec![];

        for (i, sentence) in sentences.iter().enumerate() {
            log::info!("[{}] {}", i, sentence);
            let out = dir.join(format!("{}_{}.wav", task_id.hyphenated().to_string(), i));
            let query = match ctx.voicevox.query(sentence, &speaker).await {
                Ok(query) => query,
                Err(e) => {
                    log::error!("Failed to query: {}", e);
                    continue;
                }
            };
            match ctx.voicevox.synthesis(query, &speaker, &out).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to synthesis: {}", e);
                    continue;
                }
            };
            artifacts.push(out.clone());
        }
        if artifacts.is_empty() {
            anyhow::bail!("Failed to synthesis");
        }

        let out = dir.join(format!("{}.wav", task_id.hyphenated().to_string()));
        let res = concat_wavs(&mut artifacts, &text_path, &out).await;
        for path in &artifacts {
            if path.exists() {
                fs::remove_file(path).unwrap();
                log::info!("Removed: {}", path.display());
            }
        }
        res?;
        Ok(None)
    }
}

async fn concat_wavs(
    artifacts: &[PathBuf],
    text_path: &PathBuf,
    out: &PathBuf,
) -> anyhow::Result<()> {
    let text = artifacts
        .iter()
        .map(|p| format!("file '{}'", p.file_name().unwrap().to_str().unwrap()))
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(text_path)?;
    f.write_all(text.as_bytes())?;

    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        text_path.display().to_string().as_str(),
        out.display().to_string().as_str(),
    ]);
    // cmd.spawn()?.wait();
    let res = cmd.output().await?;
    fs::remove_file(text_path)?;
    if !res.status.success() {
        anyhow::bail!("Failed to concat wavs: {}", String::from_utf8(res.stderr)?);
    }
    Ok(())
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
