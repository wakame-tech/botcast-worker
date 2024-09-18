use clap::Parser;
use scriper::{html2md::Html2MdExtractor, Extractor};
use worker::tasks::{client, fetch_content};

#[derive(Debug, clap::Parser)]
struct Args {
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = client();
    let args = Args::try_parse()?;
    let html = fetch_content(client, args.url.to_string()).await?;
    println!("{}", html);
    let md = Html2MdExtractor::extract(&html)?;
    println!("{}", md);
    Ok(())
}
