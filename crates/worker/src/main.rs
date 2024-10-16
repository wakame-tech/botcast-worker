use worker::{api::start_api, worker::start_worker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    start_worker();
    start_api().await
}
