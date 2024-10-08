use clap::Parser;
use readable_text::{html2md::Html2MdExtractor, Extractor};
use std::fs::OpenOptions;
use std::io::Write;
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
    let mut out_html = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.html")?;
    writeln!(out_html, "{}", html)?;
    let md = Html2MdExtractor::extract(&html)?;
    let mut out_md = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.md")?;
    writeln!(out_md, "{}", md)?;
    Ok(())
}
