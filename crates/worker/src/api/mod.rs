use crate::usecase::provider::Provider;
use router::routers;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod error;
mod router;

#[derive(Debug)]
struct AppState(Arc<Provider>);

pub async fn start_api(provider: Arc<Provider>) -> anyhow::Result<()> {
    let state = Arc::new(AppState(provider));
    let router = routers()
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    tracing::info!("Listen port: {}", port);
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
