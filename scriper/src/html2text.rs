use crate::Extractor;
use html_parser::{Dom, Element, Node};
use std::collections::HashSet;

pub struct Html2TextExtractor;

impl Extractor for Html2TextExtractor {
    fn extract(html: &str) -> anyhow::Result<String> {
        let dom = parse(html)?;
        let e = find_tag(&dom.children, "body").ok_or_else(|| anyhow::anyhow!("Body not found"))?;
        let content = collect_text(
            &e.children,
            &HashSet::from_iter(["h1", "h2", "h3", "h4", "h5", "h6", "p"]),
            false,
        );
        let content = html2text::from_read(content.as_bytes(), 80)
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("");
        Ok(content)
    }
}

fn parse(html: &str) -> anyhow::Result<Dom> {
    let pos = html
        .find("<html")
        .ok_or(anyhow::anyhow!("HTML tag not found"))?;
    let html = &html[pos..];
    Ok(Dom::parse(html)?)
}

fn find_tag(nodes: &[Node], tag: &str) -> Option<Element> {
    for n in nodes.iter() {
        if let Node::Element(e) = n {
            if e.name == tag {
                return Some(e.to_owned());
            }
            let Some(e) = find_tag(&e.children, tag) else {
                continue;
            };
            return Some(e);
        }
    }
    None
}

fn collect_text(nodes: &[Node], tags: &HashSet<&str>, in_tags: bool) -> String {
    nodes
        .iter()
        .filter_map(|n| match n {
            Node::Text(t) if in_tags => Some(t.to_owned()),
            Node::Element(e) => Some(collect_text(
                &e.children,
                tags,
                in_tags || tags.contains(&e.name.as_str()),
            )),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
fn get_title(dom: &Dom) -> anyhow::Result<String> {
    let e = find_tag(&dom.children, "title").ok_or_else(|| anyhow::anyhow!("Title not found"))?;
    let title = e
        .children
        .first()
        .unwrap()
        .text()
        .ok_or_else(|| anyhow::anyhow!("Title not found"))?;
    Ok(title.to_string())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read, path::PathBuf};

    use crate::{
        html2text::{get_title, parse, Html2TextExtractor},
        Extractor,
    };

    fn read_html(path: &str) -> anyhow::Result<String> {
        let mut f = File::open(PathBuf::from(path))?;
        let mut html = String::new();
        f.read_to_string(&mut html)?;
        Ok(html)
    }

    #[test]
    fn test_get_title() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let dom = parse(&html)?;
        let title = get_title(&dom)?;
        println!("{}", title);
        Ok(())
    }

    #[test]
    fn test_get_content() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let content = Html2TextExtractor::extract(&html)?;
        println!("content={}", content);
        Ok(())
    }
}
