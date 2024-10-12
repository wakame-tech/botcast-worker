use anyhow::Result;
use audio_generator::{voicevox::client::VoiceVoxClient, AudioGenerator};

pub(crate) fn resolve_audio_generator(resource: &str) -> Result<Box<dyn AudioGenerator>> {
    match resource {
        "voicevox" => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        _ => anyhow::bail!("Unsupported generator: {}", resource),
    }
}
