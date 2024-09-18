use s3::{creds::Credentials, Bucket, Region};

pub(crate) trait Storage {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()>;
    fn get_endpoint(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct R2Storage {
    bucket_endpoint: String,
    bucket: Box<Bucket>,
}

impl R2Storage {
    const BUCKET_NAME: &'static str = "botcast";

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
}

impl Storage for R2Storage {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()> {
        self.bucket
            .put_object_with_content_type(path, data, content_type)
            .await?;
        Ok(())
    }

    fn get_endpoint(&self) -> String {
        self.bucket_endpoint.clone()
    }
}

pub(crate) struct DummyStorage;

impl Storage for DummyStorage {
    async fn upload(&self, _path: &str, _data: &[u8], _content_type: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_endpoint(&self) -> String {
        "dummy".to_string()
    }
}
