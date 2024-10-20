use axum::async_trait;
use repos::provider::Provider;
use s3::{creds::Credentials, Bucket, Region};
use std::sync::Arc;

#[async_trait]
pub(crate) trait Storage: Send + Sync {
    async fn upload(&self, path: &str, data: &[u8], content_type: &str) -> anyhow::Result<()>;
    fn get_endpoint(&self) -> String;
}

pub(crate) trait ProviderStorage {
    fn storage(&self) -> Arc<dyn Storage>;
}

impl ProviderStorage for Provider {
    fn storage(&self) -> Arc<dyn Storage> {
        Arc::new(R2Storage::new().expect("Failed to create storage"))
    }
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

#[async_trait]
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

        fn get_endpoint(&self) -> String {
            "dummy".to_string()
        }
    }
}
