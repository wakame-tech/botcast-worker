use axum::Router;
use worker::{api::create_router, worker::start_worker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    start_worker();

    let router = Router::new();
    let app = create_router(router);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
