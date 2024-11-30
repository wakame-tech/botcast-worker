use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::str::FromStr;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use worker::{api::start_api, usecase::Provider, worker::start_worker};

fn init_tracing(otlp_collector_endpoint: String) -> anyhow::Result<()> {
    let crate_name = env!("CARGO_CRATE_NAME");

    let subscriber = tracing_subscriber::registry();
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_collector_endpoint)
        .build()?;
    // let exporter = opentelemetry_stdout::SpanExporter::default();

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            crate_name.to_string(),
        )]))
        .build();
    let otel_layer = OpenTelemetryLayer::new(tracer_provider.tracer("worker"));
    let subscriber = subscriber
        .with(otel_layer)
        .with(EnvFilter::from_str(&format!("info,{}=trace", crate_name,))?);

    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let subscriber = subscriber.with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let otlp_collector_endpoint = std::env::var("OTLP_COLLECTOR_ENDPOINT")?;
    init_tracing(otlp_collector_endpoint)?;

    let provider = Provider::default();
    start_worker(provider);
    start_api(provider).await
}
