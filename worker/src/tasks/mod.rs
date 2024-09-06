use crate::api::Args;
use anyhow::Context;
use episode_repo::EpisodeRepo;
use scriper::extractor::HtmlExtractor;
use std::{fs::File, io::Read};
use storage::Storage;
use uuid::Uuid;
use voicevox_client::{concat_wavs, VoiceVox, VoiceVoxSpeaker};
use workdir::WorkDir;

mod episode;
pub(crate) mod episode_repo;
pub(crate) mod storage;
pub(crate) mod voicevox_client;
mod workdir;

static USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36";

pub(crate) async fn run<E: EpisodeRepo, S: Storage>(
    episode_repo: &E,
    storage: &S,
    task_id: Uuid,
    args: &Args,
) -> anyhow::Result<()> {
    let Some(mut episode) = episode_repo
        .find_by_id(&Uuid::parse_str(&args.episode_id)?)
        .await?
    else {
        return Err(anyhow::anyhow!("Episode not found"));
    };

    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let res = client.get(&args.url).send().await?;
    if res.status() != reqwest::StatusCode::OK {
        anyhow::bail!("Failed to fetch: {}", res.status());
    }
    res.headers().get("content-type");
    let html = res.text().await?;

    let extractor = HtmlExtractor::new(html)?;
    let title = extractor.get_title().context("Failed to get title")?;
    let content = extractor.get_content().context("Failed to get content")?;
    log::info!("Scraped: {} {} B", episode.title, content.len());

    episode.title = title;
    episode.content = Some(content);
    episode_repo.update(&episode).await?;

    let text = episode
        .content
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

    let sentences = text.split('。').collect::<Vec<_>>();
    if sentences.is_empty() {
        anyhow::bail!("Sentences is empty");
    }

    let work_dir = WorkDir::new(&task_id, false)?;

    let voicevox_endpoint =
        std::env::var("VOICEVOX_ENDPOINT").unwrap_or("http://localhost:50021".to_string());
    log::info!("VoiceVox endpoint: {}", voicevox_endpoint);
    let voicevox = VoiceVox::new(voicevox_endpoint);
    let speaker = VoiceVoxSpeaker::ZundaNormal;

    let mut paths = vec![];
    for (i, sentence) in sentences.iter().enumerate() {
        log::info!("[{}] {}", i, sentence);
        let sentence_wav_path = work_dir.dir().join(format!("{}.wav", i));
        paths.push(sentence_wav_path.clone());
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
    }

    let episode_wav_path = concat_wavs(&work_dir, &paths).await?;
    let mut file = File::open(&episode_wav_path)?;
    let mut audio = vec![];
    file.read_to_end(&mut audio)?;

    let upload_path = format!("episodes/{}.wav", episode.id.hyphenated().to_string());
    storage.upload(&upload_path, &audio, "audio/wav").await?;
    episode.audio_url = Some(format!("{}/{}", storage.get_endpoint(), upload_path));
    episode_repo.update(&episode).await?;
    Ok(())
}
