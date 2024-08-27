use crate::{
    task::{RunTask, Task},
    voicevox_client::VoiceVoxSpeaker,
    Ctx,
};
use std::{fs::OpenOptions, io::Write, path::PathBuf};
use surrealdb::opt::RecordId;
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Synthesis {
    #[serde(skip_serializing)]
    pub(crate) id: RecordId,
    pub(crate) text: String,
    pub(crate) speaker: VoiceVoxSpeaker,
    pub(crate) out: PathBuf,
    pub(crate) artifacts: Vec<PathBuf>,
}

impl RunTask for Synthesis {
    fn id(&self) -> &RecordId {
        &self.id
    }

    async fn run(&mut self, ctx: &Ctx) -> anyhow::Result<Option<Task>> {
        let sentences = self.text.split('。').collect::<Vec<_>>();
        let dir = PathBuf::from("temp");
        if !dir.exists() {
            std::fs::create_dir(&dir)?;
        }
        let text_path = dir.join("text.txt");

        for (i, sentence) in sentences.iter().enumerate() {
            log::info!("[{}] {}", i, sentence);
            let out = dir.join(format!("{}_{}.wav", self.id.id.to_raw().to_string(), i));
            let query = match ctx.voicevox.query(sentence, &self.speaker).await {
                Ok(query) => query,
                Err(e) => {
                    log::error!("Failed to query: {}", e);
                    continue;
                }
            };
            match ctx.voicevox.synthesis(query, &self.speaker, &out).await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to synthesis: {}", e);
                    continue;
                }
            };
            self.artifacts.push(out.clone());
        }
        let out = dir.join(format!("{}.wav", self.id.id.to_raw().to_string()));
        concat_wavs(&mut self.artifacts, &text_path, &out).await?;
        Ok(None)
    }
}

async fn concat_wavs(
    artifacts: &mut Vec<PathBuf>,
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
    artifacts.push(text_path.clone());

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
    if !res.status.success() {
        anyhow::bail!("Failed to concat wavs: {}", String::from_utf8(res.stderr)?);
    }
    Ok(())
}

impl Drop for Synthesis {
    fn drop(&mut self) {
        for path in &self.artifacts {
            if path.exists() {
                std::fs::remove_file(path).unwrap();
                log::info!("Removed: {}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::voicevox_client::{VoiceVox, VoiceVoxSpeaker};
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
