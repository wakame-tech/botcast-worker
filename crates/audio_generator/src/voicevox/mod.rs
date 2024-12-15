pub mod client;

use crate::{ffmpeg::convert_to_stereo_wav, workdir::WorkDir, AudioGenerator};
use anyhow::Result;
use api::episode::Section;
use async_trait::async_trait;
use client::VoiceVoxClient;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};
use tracing::instrument;
use wavers::Wav;

const DELIMITERS: [char; 2] = ['\n', '。'];

fn split_text(text: &str, size: usize) -> Vec<String> {
    let mut buf = String::new();
    let mut res = vec![];
    for sentence in text.split_inclusive(DELIMITERS) {
        let sentence = sentence.trim();
        if sentence.is_empty() || sentence.starts_with("http") {
            continue;
        }
        if buf.len() + sentence.len() > size {
            res.push(buf.clone());
            buf.clear();
        }
        buf.push_str(sentence);
    }
    if !buf.is_empty() {
        res.push(buf.clone());
        buf.clear();
    }
    res
}

#[async_trait]
impl AudioGenerator for VoiceVoxClient {
    #[instrument(skip(self), ret)]
    async fn generate(
        &self,
        i: &mut usize,
        work_dir: &WorkDir,
        section: Section,
    ) -> Result<Vec<(PathBuf, String)>> {
        let mut path_and_texts = vec![];

        let Section::Serif { text, speaker } = section else {
            return Err(anyhow::anyhow!("Invalid segment"));
        };

        for sentence in split_text(&text, 100) {
            let wav_path = work_dir.dir().join(format!("{}.wav", i));
            tracing::info!("[{}]: {}", i, sentence);
            *i += 1;
            if wav_path.exists() {
                path_and_texts.push((wav_path, sentence));
                continue;
            }

            let query = self.query(&sentence, &speaker).await?;
            let audio = self.synthesis(query, &speaker).await?;

            let mut f = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&wav_path)?;
            f.write_all(&audio)?;

            let file = Box::new(f);
            let mut r = Wav::<i16>::new(file)?;
            if r.channels().count() == 1 {
                let tmp = work_dir.dir().join("tmp.wav");
                convert_to_stereo_wav(&wav_path, &tmp)?;
                fs::rename(tmp, &wav_path)?;
            }
            path_and_texts.push((wav_path, sentence));
        }

        Ok(path_and_texts)
    }
}
