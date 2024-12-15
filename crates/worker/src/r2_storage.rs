use axum::async_trait;
use repos::provider::DefaultProvider;
use s3::{creds::Credentials, Bucket, Region};
use std::{fmt::Debug, sync::Arc};

#[async_trait]
pub(crate) trait Storage: Send + Sync {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()>;
}

pub(crate) trait ProvideStorage: Debug + Send + Sync {
    fn storage(&self) -> Arc<dyn Storage>;
}

impl ProvideStorage for DefaultProvider {
    fn storage(&self) -> Arc<dyn Storage> {
        Arc::new(R2Storage::new().expect("Failed to create storage"))
    }
}

#[derive(Debug, Clone)]
pub struct R2Storage {
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
        Ok(Self { bucket })
    }
}

#[async_trait]
impl Storage for R2Storage {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()> {
        self.bucket
            .put_object_with_content_type(path, data, content_type)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    pub(crate) struct DummyStorage;

    #[async_trait]
    impl Storage for DummyStorage {
        async fn upload(
            &self,
            _path: &str,
            _data: &[u8],
            _content_type: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
