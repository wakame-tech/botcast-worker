pub mod html2md;
pub mod html2text;

pub trait Extractor {
    fn extract(html: &str) -> anyhow::Result<String>;
}
