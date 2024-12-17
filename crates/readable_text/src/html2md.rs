use html2md::{dummy::DummyHandler, TagHandler, TagHandlerFactory};
use std::collections::HashMap;

pub struct ReadableText;

impl ReadableText {
    pub fn extract(html: &str) -> anyhow::Result<String> {
        let md = parse_to_md(html);
        Ok(md)
    }
}

fn parse_to_md(html: &str) -> String {
    let mut handlers = HashMap::<String, Box<dyn TagHandlerFactory>>::new();
    handlers.insert("head".to_string(), Box::new(IgnoreHandlerFactory));
    handlers.insert("h1".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("h2".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("h3".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("h4".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("h5".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("h6".to_string(), Box::new(HeaderHandlerFactory));
    handlers.insert("a".to_string(), Box::new(DummyHandlerFactory));
    handlers.insert("em".to_string(), Box::new(DummyHandlerFactory));
    handlers.insert("strong".to_string(), Box::new(DummyHandlerFactory));
    handlers.insert("img".to_string(), Box::new(IgnoreHandlerFactory));
    handlers.insert("table".to_string(), Box::new(DummyHandlerFactory));
    handlers.insert("nav".to_string(), Box::new(DummyHandlerFactory));
    handlers.insert("pre".to_string(), Box::new(IgnoreHandlerFactory));
    handlers.insert("script".to_string(), Box::new(IgnoreHandlerFactory));
    handlers.insert("style".to_string(), Box::new(IgnoreHandlerFactory));
    handlers.insert("iframe".to_string(), Box::new(IgnoreHandlerFactory));
    
    html2md::parse_html_custom(html, &handlers)
}

struct IgnoreHandlerFactory;

struct IgnoreHandler;

impl TagHandler for IgnoreHandler {
    fn handle(&mut self, _tag: &html2md::Handle, _printer: &mut html2md::StructuredPrinter) {}

    fn skip_descendants(&self) -> bool {
        true
    }

    fn after_handle(&mut self, _printer: &mut html2md::StructuredPrinter) {}
}

impl TagHandlerFactory for IgnoreHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(IgnoreHandler)
    }
}

struct DummyHandlerFactory;

impl TagHandlerFactory for DummyHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(DummyHandler)
    }
}

struct HeaderHandler;

impl TagHandler for HeaderHandler {
    fn handle(&mut self, tag: &html2md::Handle, printer: &mut html2md::StructuredPrinter) {
        html2md::headers::HeaderHandler::handle(
            &mut html2md::headers::HeaderHandler::default(),
            tag,
            printer,
        );
    }

    fn after_handle(&mut self, _printer: &mut html2md::StructuredPrinter) {}
}

struct HeaderHandlerFactory;

impl TagHandlerFactory for HeaderHandlerFactory {
    fn instantiate(&self) -> Box<dyn TagHandler> {
        Box::new(HeaderHandler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract() {
        for (html, expected_md) in [
            (r#"<em>hoge</em>"#, "hoge"),
            (r#"<strong>hoge</strong>"#, "hoge"),
            (r#"<img src="a"></img>"#, ""),
            (r#"<a href="http://example.com">hoge</a>"#, "hoge"),
        ] {
            let md = ReadableText::extract(html).unwrap();
            assert_eq!(md, expected_md);
        }
    }
}
