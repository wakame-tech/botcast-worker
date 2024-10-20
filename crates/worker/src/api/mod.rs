use repos::provider::Provider;
use router::routers;
use std::sync::Arc;

mod error;
mod router;

#[derive(Debug, Clone)]
struct AppState(Provider);

pub async fn start_api(provider: Provider) -> anyhow::Result<()> {
    let state = Arc::new(AppState(provider));
    let router = routers().with_state(state);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
