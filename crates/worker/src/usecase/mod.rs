pub(crate) mod episode_service;
pub(crate) mod provider;
pub(crate) mod script_service;
pub(crate) mod task_service;

use api::client::ApiClient;
pub use provider::Provider;
use repos::provider::DefaultProvider;
use std::sync::Arc;

pub(crate) trait ProvideApiClient {
    fn api_client(&self) -> Arc<ApiClient>;
}

impl ProvideApiClient for DefaultProvider {
    fn api_client(&self) -> Arc<ApiClient> {
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .expect("SUPABASE_SERVICE_ROLE_KEY is not set");
        let api_endpoint = std::env::var("API_ENDPOINT").expect("API_ENDPOINT is not set");
        Arc::new(ApiClient::new(&api_endpoint, &service_role_key))
    }
}
