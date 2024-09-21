use crate::api::Args;
use encoding::{all::UTF_8, DecoderTrap, Encoding};
use episode_repo::EpisodeRepo;
use reqwest::Client;
use scriper::{html2md::Html2MdExtractor, Extractor};
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

pub(crate) async fn run<E: EpisodeRepo, S: Storage>(
    episode_repo: &E,
    storage: &S,
    task_id: Uuid,
    args: &Args,
) -> anyhow::Result<()> {
    let keep = std::env::var("KEEP_WORKDIR")
        .unwrap_or("false".to_string())
        .parse()?;
    let work_dir = WorkDir::new(&task_id, keep)?;

    let Some(mut episode) = episode_repo
        .find_by_id(&Uuid::parse_str(&args.episode_id)?)
        .await?
    else {
        return Err(anyhow::anyhow!("Episode not found"));
    };

    let client = client();
    let html = fetch_content(client, args.url.to_string()).await?;
    let content = Html2MdExtractor::extract(&html)?;
    let mut content_file = File::create(work_dir.dir().join("content.md"))?;
    write!(content_file, "# {}\n\n", episode.title)?;
    content_file.write_all(content.as_bytes())?;

    log::info!("Scraped: {} {} B", episode.title, content.len());

    episode.content = Some(content);
    episode_repo.update(&episode).await?;

    let text = episode
        .content
        .ok_or_else(|| anyhow::anyhow!("Content not found"))?;

    let sentences = text.split_inclusive(['。', '\n']).collect::<Vec<_>>();
    if sentences.is_empty() {
        anyhow::bail!("Sentences is empty");
    }

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

    let mut content_with_timestamp = String::new();
    let mut duration = Duration::ZERO;
    for (path, sentence) in &paths {
        let mmss = format!(
            "{:02}:{:02}",
            duration.as_secs() / 60,
            duration.as_secs() % 60
        );
        content_with_timestamp += &format!("[{}](#{}) {}\n", mmss, mmss, sentence);

        log::info!("{}: {}", mmss, sentence);
        let file = Box::new(File::open(path)?);
        let file: Wav<i16> = Wav::new(file)?;
        duration += get_duration(&file);
    }

    episode.content = Some(content_with_timestamp);
    episode_repo.update(&episode).await?;

    let paths = paths.into_iter().map(|(path, _)| path).collect::<Vec<_>>();
    let episode_audio_path = concat_audios(&work_dir, &paths).await?;

    let mut file = File::open(&episode_audio_path)?;
    let mut audio = vec![];
    file.read_to_end(&mut audio)?;

    let upload_path = format!("episodes/{}.mp3", episode.id.hyphenated());
    storage.upload(&upload_path, &audio, "audio/mp3").await?;
    episode.audio_url = Some(format!("{}/{}", storage.get_endpoint(), upload_path));
    episode_repo.update(&episode).await?;
    Ok(())
}
