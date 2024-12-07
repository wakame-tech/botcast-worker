use super::workdir::WorkDir;
use std::{fs::OpenOptions, io::Write, path::PathBuf, time::Duration};
use tokio::process::Command;
use tracing::instrument;
use wavers::Wav;

pub(crate) fn get_duration(wav: &Wav<i16>) -> Duration {
    let data_size = wav.header().data().size;

    let sample_rate = wav.sample_rate() as u32;
    let n_channels = wav.n_channels() as u32;
    let bytes_per_sample = (wav.header().fmt_chunk.bits_per_sample / 8) as u32;

    Duration::from_secs_f32(data_size as f32 / (sample_rate * n_channels * bytes_per_sample) as f32)
}

pub(crate) async fn slice_audio(
    input: &PathBuf,
    output: &PathBuf,
    from_sec: f64,
    duration_sec: f64,
) -> anyhow::Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-ss",
        from_sec.to_string().as_str(),
        "-to",
        duration_sec.to_string().as_str(),
        "-i",
        input.display().to_string().as_str(),
        output.display().to_string().as_str(),
    ]);
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!("Failed to slice audio: {}", String::from_utf8(res.stderr)?);
    }
    Ok(())
}

#[instrument(skip(work_dir, paths))]
pub(crate) async fn concat_audios(
    work_dir: &WorkDir,
    paths: &[PathBuf],
) -> anyhow::Result<PathBuf> {
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

    let episode_audio_path = work_dir.dir().join("episode.mp3");
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        inputs_path.display().to_string().as_str(),
        "-vn",
        "-ar",
        "44100",
        "-ac",
        "2",
        "-b:a",
        "192k",
        episode_audio_path.display().to_string().as_str(),
    ]);
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!(
            "Failed to concat audios: {}",
            String::from_utf8(res.stderr)?
        );
    }
    Ok(episode_audio_path)
}
