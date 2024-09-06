use s3::{creds::Credentials, Bucket, Region};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct R2Client {
    bucket_endpoint: String,
    bucket: Box<Bucket>,
}

impl R2Client {
    const BUCKET_NAME: &'static str = "botcast";
    const DIRECTORY_NAME: &'static str = "episodes";

    pub fn new() -> anyhow::Result<Self> {
        let bucket = Bucket::new(
            Self::BUCKET_NAME,
            Region::R2 {
                account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID")?,
            },
            Credentials::from_env()?,
        )?;
        Ok(Self {
            bucket_endpoint: std::env::var("BUCKET_ENDPOINT")?,
            bucket,
        })
    }

    pub async fn upload_wav(&self, id: &Uuid, data: &[u8]) -> anyhow::Result<()> {
        self.bucket
            .put_object_with_content_type(
                format!(
                    "{}/{}.wav",
                    Self::DIRECTORY_NAME,
                    id.hyphenated().to_string()
                ),
                data,
                "audio/wav",
            )
            .await?;
        Ok(())
    }

    pub fn get_url(&self, id: &Uuid) -> String {
        format!(
            "{}/{}/{}.wav",
            self.bucket_endpoint,
            Self::DIRECTORY_NAME,
            id.hyphenated().to_string()
        )
    }
}
