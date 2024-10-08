use worker::{api::start_api, app_module::AppModule, worker::start_worker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let app_module = AppModule::new().await;
    start_worker(app_module.clone());
    start_api(app_module).await
}
