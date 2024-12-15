use crate::{
    audio_downloader::AudioDownloader,
    ffmpeg::{concat_audios, get_duration},
    voicevox::client::VoiceVoxClient,
    workdir::WorkDir,
    AudioGenerator,
};
use anyhow::Result;
use api::episode::Section;
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{fs::File, path::PathBuf, time::Duration};
use wavers::Wav;

fn resolve_audio_generator(section: &Section) -> Result<Box<dyn AudioGenerator>> {
    match section {
        Section::Serif { .. } => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        Section::Audio { .. } => Ok(Box::new(AudioDownloader::new())),
    }
}

pub struct SynthesisResult {
    pub out_path: PathBuf,
    pub srt: String,
    pub duration_sec: f64,
}

pub async fn generate_audio(
    work_dir: &WorkDir,
    sections: Vec<Section>,
) -> anyhow::Result<SynthesisResult> {
    let mut subs = vec![];

    let mut i = 0;
    for section in sections {
        let generator = resolve_audio_generator(&section)?;
        let paths = generator
            .generate(&mut i, work_dir, section.clone())
            .await?;
        subs.extend(paths);
    }

    let mut srt = Subtitles::new();
    let mut duration = Duration::ZERO;

    let mmss = |d: &Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
    for (i, (path, sentence)) in subs.iter().enumerate() {
        let file = Box::new(File::open(path)?);
        let r = Wav::<i16>::new(file)?;
        let (start, end) = (duration, duration + get_duration(&r));
        tracing::info!("{} -> {}: {}", mmss(&start), mmss(&end), sentence);
        let sub = Subtitle::new(
            i,
            Timestamp::from_milliseconds(start.as_millis() as u32),
            Timestamp::from_milliseconds(end.as_millis() as u32),
            sentence.to_string(),
        );
        srt.push(sub);
        duration = end;
    }

    let paths = subs.into_iter().map(|(path, _)| path).collect::<Vec<_>>();
    let episode_audio_path = concat_audios(work_dir, &paths).await?;

    Ok(SynthesisResult {
        out_path: episode_audio_path,
        srt: srt.to_string(),
        duration_sec: duration.as_secs_f64(),
    })
}
