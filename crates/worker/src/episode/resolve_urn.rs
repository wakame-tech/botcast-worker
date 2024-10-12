use anyhow::Result;
use script_audio_generator::{voicevox::api::VoiceVoxClient, AudioGenerator};
use script_runtime::Urn;

pub fn resolve_urn(urn: &Urn) -> Result<(String, String)> {
    let [sig, generator, speaker_id]: [&str; 3] = urn
        .0
        .splitn(3, ':')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid URN: {}", urn.0))?;
    anyhow::ensure!(sig == "urn", "Invalid URN: {}", urn.0);
    Ok((generator.to_string(), speaker_id.to_string()))
}

pub fn resolve_generator(generator: &str) -> Result<Box<dyn AudioGenerator>> {
    match generator {
        "voicevox" => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        _ => anyhow::bail!("Unsupported generator: {}", generator),
    }
}
