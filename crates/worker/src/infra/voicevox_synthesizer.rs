use crate::infra::{
    ffmpeg::{concat_audios, get_duration},
    voicevox_client::{VoiceVoxClient, VoiceVoxSpeaker},
    workdir::WorkDir,
};
use axum::async_trait;
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{fs::File, path::PathBuf, time::Duration};
use wavers::Wav;

pub(crate) struct SynthesisResult {
    pub(crate) out_path: PathBuf,
    pub(crate) srt: String,
}

#[async_trait]
pub(crate) trait AudioSynthesizer: Send + Sync {
    async fn synthesis_sentences(
        &self,
        work_dir: &WorkDir,
        sentences: Vec<String>,
    ) -> anyhow::Result<SynthesisResult>;
}

pub(crate) struct VoiceVoxAudioSynthesizer {
    client: VoiceVoxClient,
    speaker: VoiceVoxSpeaker,
}

impl Default for VoiceVoxAudioSynthesizer {
    fn default() -> Self {
        Self {
            client: VoiceVoxClient::new(),
            speaker: VoiceVoxSpeaker::ZundaNormal,
        }
    }
}

#[async_trait]
impl AudioSynthesizer for VoiceVoxAudioSynthesizer {
    async fn synthesis_sentences(
        &self,
        work_dir: &WorkDir,
        sentences: Vec<String>,
    ) -> anyhow::Result<SynthesisResult> {
        let mut paths = vec![];
        for (i, sentence) in sentences.iter().enumerate() {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            let sentence_wav_path = work_dir.dir().join(format!("{}.wav", i));
            let query = match self.client.query(sentence, &self.speaker).await {
                Ok(query) => query,
                Err(e) => {
                    log::error!("Failed to query: {}", e);
                    continue;
                }
            };
            match self
                .client
                .synthesis(query, &self.speaker, &sentence_wav_path)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to synthesis: {}", e);
                    continue;
                }
            };

            if sentence_wav_path.exists() {
                paths.push((sentence_wav_path, sentence));
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
}
