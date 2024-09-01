use html_parser::{Dom, Element, Node};
use std::collections::HashSet;

#[derive(Debug)]
pub(crate) struct Extractor {
    html: Dom,
}

impl Extractor {
    pub(crate) fn new(html: String) -> anyhow::Result<Self> {
        let pos = html
            .find("<html")
            .ok_or(anyhow::anyhow!("HTML tag not found"))?;
        let html = &html[pos..];
        let html = Dom::parse(&html)?;
        Ok(Self { html })
    }

    fn find_tag(nodes: &[Node], tag: &str) -> Option<Element> {
        for n in nodes.iter() {
            match n {
                Node::Element(e) => {
                    if e.name == tag {
                        return Some(e.to_owned());
                    }
                    let Some(e) = Self::find_tag(&e.children, tag) else {
                        continue;
                    };
                    return Some(e);
                }
                _ => {}
            }
        }
        None
    }

    fn collect_text(nodes: &[Node], tags: &HashSet<&str>, in_tags: bool) -> String {
        nodes
            .iter()
            .filter_map(|n| match n {
                Node::Text(t) if in_tags => Some(t.to_owned()),
                Node::Element(e) => Some(Self::collect_text(
                    &e.children,
                    tags,
                    tags.contains(&e.name.as_str()),
                )),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) fn get_title(&self) -> anyhow::Result<String> {
        let e = Self::find_tag(&self.html.children, "title")
            .ok_or_else(|| anyhow::anyhow!("Title not found"))?;
        let title = e
            .children
            .get(0)
            .unwrap()
            .text()
            .ok_or_else(|| anyhow::anyhow!("Title not found"))?;
        Ok(title.to_string())
    }

    pub(crate) fn get_content(&self) -> anyhow::Result<String> {
        let e = Self::find_tag(&self.html.children, "body")
            .ok_or_else(|| anyhow::anyhow!("Body not found"))?;
        let content = Self::collect_text(&e.children, &HashSet::from_iter(["p"]), false);
        let content = html2text::from_read(content.as_bytes(), 80)
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use crate::worker::extractor::Extractor;
    use std::{fs::File, io::Read, path::PathBuf};

    fn read_html<'a>(path: &str) -> anyhow::Result<String> {
        let mut f = File::open(PathBuf::from(path))?;
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        Ok(html)
    }

    #[test]
    fn test_get_title() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let extractor = Extractor::new(html)?;
        let title = extractor.get_title()?;
        println!("{}", title);
        Ok(())
    }

    #[test]
    fn test_get_content() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let extractor = Extractor::new(html)?;
        let content = extractor.get_content()?;
        println!("content={}", content);
        Ok(())
    }
}
