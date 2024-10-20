use crate::{
    ffmpeg::{concat_audios, get_duration},
    voicevox::client::VoiceVoxClient,
    workdir::WorkDir,
    AudioGenerator,
};
use anyhow::Result;
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{fs::File, io::Write, path::PathBuf, time::Duration};
use wavers::Wav;

fn resolve_audio_generator(resource: &str) -> Result<Box<dyn AudioGenerator>> {
    match resource {
        "voicevox" => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        _ => anyhow::bail!("Unsupported generator: {}", resource),
    }
}

#[derive(Debug)]
pub struct Sentence {
    speaker: (String, String),
    text: String,
}

impl Sentence {
    pub fn new(speaker: (String, String), text: String) -> Self {
        Self { speaker, text }
    }
}

pub struct SynthesisResult {
    pub out_path: PathBuf,
    pub srt: String,
}

pub async fn generate_audio(
    work_dir: &WorkDir,
    sentences: &[Sentence],
) -> anyhow::Result<SynthesisResult> {
    let mut paths = vec![];
    for (
        i,
        Sentence {
            speaker: (generator, speaker_id),
            text,
        },
    ) in sentences.iter().enumerate()
    {
        let generator = resolve_audio_generator(&generator)?;
        let sentence_wav_path = work_dir.dir().join(format!("{}.wav", i));
        let wav = generator.generate(speaker_id, text).await?;
        let mut sentence_wav = File::create(&sentence_wav_path)?;
        sentence_wav.write_all(&wav)?;

        if sentence_wav_path.exists() {
            paths.push((sentence_wav_path, text));
        }
    }

    let mut subs = Subtitles::new();
    let mut duration = Duration::ZERO;

    let mmss = |d: &Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
    for (i, (path, sentence)) in paths.iter().enumerate() {
        let file = Box::new(File::open(path)?);
        let file: Wav<i16> = Wav::new(file)?;
        let (start, end) = (duration, duration + get_duration(&file));
        log::info!("{} -> {}: {}", mmss(&start), mmss(&end), sentence);
        let sub = Subtitle::new(
            i,
            Timestamp::from_milliseconds(start.as_millis() as u32),
            Timestamp::from_milliseconds(end.as_millis() as u32),
            sentence.to_string(),
        );
        subs.push(sub);
        duration = end;
    }

    let paths = paths.into_iter().map(|(path, _)| path).collect::<Vec<_>>();
    let episode_audio_path = concat_audios(work_dir, &paths).await?;

    Ok(SynthesisResult {
        out_path: episode_audio_path,
        srt: subs.to_string(),
    })
}
