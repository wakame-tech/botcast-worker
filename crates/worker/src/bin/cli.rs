use clap::Parser;
use readable_text::{html2md::Html2MdExtractor, Extractor};
use worker::infra::http_client::HttpClient;

#[derive(Debug, clap::Parser)]
struct Args {
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = HttpClient::default();
    let args = Args::try_parse()?;
    let html = client.fetch_content_as_utf8(args.url.parse()?).await?;
    println!("{}", html);
    let md = Html2MdExtractor::extract(&html)?;
    println!("{}", md);
    Ok(())
}
