use clap::Parser;
use readable_text::ReadableText;
use script_http_client::HttpClient;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, clap::Parser)]
struct Args {
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = HttpClient::new(std::env::var("USER_AGENT").ok());
    let args = Args::try_parse()?;
    let html = client.fetch_content_as_utf8(args.url).await?;
    let mut out_html = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.html")?;
    writeln!(out_html, "{}", html)?;
    let md = ReadableText::extract(&html)?;
    let mut out_md = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("out.md")?;
    writeln!(out_md, "{}", md)?;
    Ok(())
}
