use crate::api::Args;
use encoding::{all::UTF_8, DecoderTrap, Encoding};
use episode_repo::EpisodeRepo;
use reqwest::Client;
use scriper::{html2md::Html2MdExtractor, Extractor};
use std::{fs::File, io::Read, sync::OnceLock};
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

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
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

    let client = client();
    let html = fetch_content(&client, args.url.to_string()).await?;
    let content = Html2MdExtractor::extract(&html)?;
    log::info!("Scraped: {} {} B", episode.title, content.len());

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

    let upload_path = format!("episodes/{}.wav", episode.id.hyphenated());
    storage.upload(&upload_path, &audio, "audio/wav").await?;
    episode.audio_url = Some(format!("{}/{}", storage.get_endpoint(), upload_path));
    episode_repo.update(&episode).await?;
    Ok(())
}
