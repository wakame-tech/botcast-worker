use anyhow::Result;
use script_audio_generator::{voicevox::api::VoiceVoxClient, AudioGenerator};
use script_runtime::Urn;

pub fn resolve_urn(urn: &Urn) -> Result<(String, String)> {
    let [sig, resource, resource_id]: [&str; 3] = urn
        .0
        .splitn(3, ':')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid URN: {}", urn.0))?;
    anyhow::ensure!(sig == "urn", "Invalid URN: {}", urn.0);
    Ok((resource.to_string(), resource_id.to_string()))
}

pub(crate) fn resolve_audio_generator(resource: &str) -> Result<Box<dyn AudioGenerator>> {
    match resource {
        "voicevox" => {
            let end_point = std::env::var("VOICEVOX_ENDPOINT")?;
            Ok(Box::new(VoiceVoxClient::new(end_point)))
        }
        _ => anyhow::bail!("Unsupported generator: {}", resource),
    }
}
