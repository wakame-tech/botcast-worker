use encoding::{all::UTF_8, DecoderTrap, Encoding};
use episode_repo::EpisodeRepo;
use reqwest::{Client, Url};
use scriper::{html2md::Html2MdExtractor, Extractor};
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{
    fs::File,
    io::{Read, Write},
    sync::OnceLock,
    time::Duration,
};
use storage::Storage;
use uuid::Uuid;
use voicevox_client::{concat_audios, get_duration, VoiceVox, VoiceVoxSpeaker};
use wavers::Wav;
use workdir::WorkDir;

mod episode;
pub(crate) mod episode_repo;
pub(crate) mod storage;
pub(crate) mod voicevox_client;
mod workdir;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(std::env::var("USER_AGENT").unwrap_or_default())
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build HTTP client")
    })
}

pub async fn fetch_content(client: &Client, url: String) -> anyhow::Result<String> {
    let res = client.get(url).send().await?;
    if res.status() != reqwest::StatusCode::OK {
        anyhow::bail!("Failed to fetch: {}", res.status());
    }
    let html = res.bytes().await?;
    let html = match xmldecl::parse(&html) {
        Some(e) => e.decode(&html).0.into_owned(),
        None => UTF_8
            .decode(&html, DecoderTrap::Strict)
            .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?,
    };
    Ok(html)
}

pub(crate) struct EpisodeService {
    pub(crate) episode_repo: Box<dyn EpisodeRepo>,
    pub(crate) storage: Box<dyn Storage>,
}

impl EpisodeService {
    fn use_work_dir(&self, task_id: &Uuid) -> anyhow::Result<WorkDir> {
        let keep = std::env::var("KEEP_WORKDIR")
            .unwrap_or("false".to_string())
            .parse()?;
        WorkDir::new(&task_id, keep)
    }

    pub(crate) async fn generate_script_from_url(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        url: Url,
    ) -> anyhow::Result<Vec<String>> {
        let work_dir = self.use_work_dir(&task_id)?;
        let Some(episode) = self.episode_repo.find_by_id(&episode_id).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };

        let client = client();
        let html = fetch_content(client, url.to_string()).await?;
        let content = Html2MdExtractor::extract(&html)?;
        let mut content_file = File::create(work_dir.dir().join("content.md"))?;
        write!(content_file, "# {}\n\n", episode.title)?;
        content_file.write_all(content.as_bytes())?;

        log::info!("Scraped: {} {} B", episode.title, content.len());

        let sentences = content
            .split_inclusive(['。', '\n'])
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if sentences.is_empty() {
            anyhow::bail!("Sentences is empty");
        }
        Ok(sentences)
    }

    pub(crate) async fn synthesis_audio(
        &self,
        task_id: Uuid,
        episode_id: Uuid,
        sentences: Vec<String>,
    ) -> anyhow::Result<()> {
        let work_dir = self.use_work_dir(&task_id)?;
        let Some(mut episode) = self.episode_repo.find_by_id(&episode_id).await? else {
            return Err(anyhow::anyhow!("Episode not found"));
        };

        let voicevox = VoiceVox::new();
        let speaker = VoiceVoxSpeaker::ZundaNormal;

        let mut paths = vec![];
        for (i, sentence) in sentences.iter().enumerate() {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            let sentence_wav_path = work_dir.dir().join(format!("{}.wav", i));
            let query = match voicevox.query(sentence, &speaker).await {
                Ok(query) => query,
                Err(e) => {
                    log::error!("Failed to query: {}", e);
                    continue;
                }
            };
            match voicevox
                .synthesis(query, &speaker, &sentence_wav_path)
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

        self.episode_repo.update(&episode).await?;

        let paths = paths.into_iter().map(|(path, _)| path).collect::<Vec<_>>();
        let episode_audio_path = concat_audios(&work_dir, &paths).await?;

        let mut file = File::open(&episode_audio_path)?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)?;

        let mp3_path = format!("episodes/{}.mp3", episode.id.hyphenated());
        self.storage.upload(&mp3_path, &audio, "audio/mp3").await?;
        episode.audio_url = Some(format!("{}/{}", self.storage.get_endpoint(), mp3_path));

        let srt_path = format!("episodes/{}.srt", episode.id.hyphenated());
        self.storage
            .upload(&srt_path, subs.to_string().as_bytes(), "text/plain")
            .await?;
        episode.script_url = Some(format!("{}/{}", self.storage.get_endpoint(), srt_path));

        self.episode_repo.update(&episode).await?;
        Ok(())
    }
}
