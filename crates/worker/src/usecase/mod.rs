pub(crate) mod episode_service;
pub(crate) mod provider;
pub(crate) mod script_service;
pub(crate) mod task_service;

use api::client::ApiClient;
pub use provider::Provider;
use std::{fmt::Debug, sync::Arc};

pub(crate) trait ProvideApiClient: Debug + Send + Sync {
    fn api_client(&self) -> Arc<ApiClient>;
}

#[derive(Debug, Default)]
pub(crate) struct UserApiClientProvider {
    user_token: Option<String>,
}

impl UserApiClientProvider {
    pub(crate) fn new(user_token: Option<String>) -> Self {
        Self { user_token }
    }
}

impl ProvideApiClient for UserApiClientProvider {
    fn api_client(&self) -> Arc<ApiClient> {
        let api_endpoint = std::env::var("API_ENDPOINT").expect("API_ENDPOINT is not set");
        Arc::new(ApiClient::new(
            api_endpoint.as_str(),
            &self.user_token.as_ref().cloned().unwrap_or_else(|| {
                std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                    .expect("SUPABASE_SERVICE_ROLE_KEY is not set")
            }),
        ))
    }
}
