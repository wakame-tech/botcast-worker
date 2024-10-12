#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VoiceVoxSpeaker {
    ZundaNormal,
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
    use crate::voicevox::speaker::VoiceVoxSpeaker;

    #[test]
    fn test_parse_speaker_id() {
        let speaker: VoiceVoxSpeaker = serde_json::from_str(r#""zunda_normal""#).unwrap();
        assert_eq!(speaker.id(), "3");
    }
}
