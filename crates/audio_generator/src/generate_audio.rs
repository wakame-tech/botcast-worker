use crate::{
    audio_downloader::AudioDownloader,
    ffmpeg::{concat_audios, convert_to_stereo_wav, get_duration},
    voicevox::client::VoiceVoxClient,
    workdir::WorkDir,
    AudioGenerator,
};
use anyhow::Result;
use reqwest::Url;
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use wavers::Wav;

fn resolve_audio_generator(segment: &SectionSegment) -> Result<Box<dyn AudioGenerator>> {
    match segment {
        SectionSegment::SerifSentence { .. } => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        SectionSegment::Audio { .. } => Ok(Box::new(AudioDownloader::new())),
    }
}

#[derive(Debug)]
pub enum SectionSegment {
    SerifSentence {
        speaker: String,
        text: String,
    },
    Audio {
        url: Url,
        from_sec: Option<f64>,
        to_sec: Option<f64>,
    },
}

pub struct SynthesisResult {
    pub out_path: PathBuf,
    pub srt: String,
    pub duration_sec: f64,
}

pub async fn generate_audio(
    work_dir: &WorkDir,
    segments: Vec<SectionSegment>,
) -> anyhow::Result<SynthesisResult> {
    let mut sentences_file = OpenOptions::new()
        .append(true)
        .create(true)
        .write(true)
        .open(work_dir.dir().join("sentences.txt"))?;
    let mut paths = vec![];
    for (i, segment) in segments.into_iter().enumerate() {
        let generator = resolve_audio_generator(&segment)?;

        let text = match &segment {
            SectionSegment::SerifSentence { text, .. } => text.clone(),
            SectionSegment::Audio { url, .. } => url.as_str().to_string(),
        };
        let wav_path = work_dir.dir().join(format!("{}.wav", i));
        if !(work_dir.is_keep_dir() && wav_path.exists()) {
            let wav = generator.generate(work_dir, segment).await?;
            let mut f = File::create(&wav_path)?;
            f.write_all(&wav)?;
        }
        if wav_path.exists() {
            paths.push((wav_path, text.clone()));
            sentences_file.write_all(format!("file '{}'\n", text).as_bytes())?;
        }
    }

    let mut subs = Subtitles::new();
    let mut duration = Duration::ZERO;

    let mmss = |d: &Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
    for (i, (path, sentence)) in paths.iter().enumerate() {
        let file = Box::new(File::open(path)?);
        let mut r = Wav::<i16>::new(file)?;
        if r.channels().count() == 1 {
            let tmp = work_dir.dir().join("tmp.wav");
            convert_to_stereo_wav(path.clone(), tmp.clone())?;
            fs::rename(tmp, path)?;
        }

        let (start, end) = (duration, duration + get_duration(&r));
        tracing::info!("{} -> {}: {}", mmss(&start), mmss(&end), sentence);
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
        duration_sec: duration.as_secs_f64(),
    })
}
