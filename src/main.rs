use api::{create_router, ctx::Ctx};
use axum::Router;
use std::sync::LazyLock;
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use worker::start_worker;

mod api;
mod episode;
mod worker;

static QUEUE_DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    QUEUE_DB.connect::<Mem>(()).await?;
    QUEUE_DB.use_ns("default").use_db("database").await?;

    let ctx = Ctx::new().await?;

    start_worker();

    let router = Router::new();
    let app = create_router(router).with_state(ctx);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;

    axum::serve(listener, app).await?;
    Ok(())
}
