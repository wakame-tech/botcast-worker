use super::{resolve_urn::resolve_audio_generator, Manuscript, Section};
use crate::infra::{
    ffmpeg::{concat_audios, get_duration},
    workdir::WorkDir,
};
use script_runtime::parse_urn;
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{fs::File, io::Write, path::PathBuf, time::Duration};
use uuid::Uuid;
use wavers::Wav;

fn use_work_dir(task_id: &Uuid) -> anyhow::Result<WorkDir> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    WorkDir::new(task_id, keep)
}

struct Sentence {
    generator: String,
    speaker_id: String,
    text: String,
}

pub(crate) struct SynthesisResult {
    pub(crate) out_path: PathBuf,
    pub(crate) srt: String,
}

pub(crate) async fn generate_audio(
    work_dir: &WorkDir,
    manuscript: &Manuscript,
) -> anyhow::Result<SynthesisResult> {
    let mut sentences = vec![];
    for section in manuscript.sections.iter() {
        match section {
            Section::Serif { text, speaker } => {
                let (resource, speaker_id) = parse_urn(speaker)?;
                for sentence in text.split(['\n', '。']) {
                    let sentence = sentence.trim();
                    if sentence.is_empty() {
                        continue;
                    }
                    sentences.push(Sentence {
                        generator: resource.clone(),
                        speaker_id: speaker_id.clone(),
                        text: text.to_string(),
                    });
                }
            }
        }
    }

    let mut paths = vec![];
    for (
        i,
        Sentence {
            generator,
            speaker_id,
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
