use std::str::FromStr;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum VoiceVoxSpeaker {
    ZundaNormal,
}

impl FromStr for VoiceVoxSpeaker {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(Into::into)
    }
}

impl VoiceVoxSpeaker {
    pub(crate) fn id(&self) -> &str {
        match self {
            Self::ZundaNormal => "3",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_speaker_id() {
        let speaker: super::VoiceVoxSpeaker = "zundanormal".parse().unwrap();
        assert_eq!(speaker.id(), "3");
    }
}
