use std::collections::HashSet;
use std::sync::LazyLock;

use linkify::{LinkFinder, LinkKind};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use regex::Regex;
use url::Url;

use super::LinkTarget;

static HTML_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<(a|img|source|video|audio|iframe)\b[^>]*>").unwrap()
});
static HTML_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
		r#"(?is)(?:^|\s)(href|src|srcset|poster)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#,
	)
	.unwrap()
});

pub(super) fn extract(markdown: &str) -> Vec<LinkTarget> {
    let mut targets = Vec::new();
    let mut code = false;
    let mut linked = 0usize;
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]).url_must_have_scheme(true);
    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::CodeBlock(_)) => code = true,
            Event::End(TagEnd::CodeBlock) => code = false,
            _ if code => {}
            Event::Start(Tag::Link { dest_url, .. }) => {
                push(&mut targets, &dest_url, false);
                linked += 1;
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                push(&mut targets, &dest_url, true);
                linked += 1;
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                linked = linked.saturating_sub(1)
            }
            Event::Text(text) if linked == 0 => {
                for link in finder.links(&text) {
                    let raw = link.as_str();
                    let url = if raw.contains("://") {
                        raw.to_owned()
                    } else {
                        format!("https://{raw}")
                    };
                    push(&mut targets, &url, false);
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                for tag in HTML_TAG.captures_iter(&html) {
                    for attr in HTML_URL.captures_iter(&tag[0]) {
                        let raw = attr
                            .get(2)
                            .or_else(|| attr.get(3))
                            .or_else(|| attr.get(4))
                            .unwrap()
                            .as_str();
                        let decoded = quick_xml::escape::unescape(raw)
                            .unwrap_or_else(|_| raw.into());
                        let image = tag[1].eq_ignore_ascii_case("img")
                            || attr[1].eq_ignore_ascii_case("poster")
                            || attr[1].eq_ignore_ascii_case("srcset");
                        if attr[1].eq_ignore_ascii_case("srcset") {
                            if !decoded.starts_with("data:") {
                                for candidate in decoded.split(',') {
                                    if let Some(url) =
                                        candidate.split_whitespace().next()
                                    {
                                        push(&mut targets, url, image);
                                    }
                                }
                            }
                        } else {
                            push(&mut targets, &decoded, image);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let mut seen = HashSet::new();
    targets.retain(|target| seen.insert((target.url.clone(), target.image)));
    targets
}

fn push(targets: &mut Vec<LinkTarget>, url: &str, image: bool) {
    if url.is_empty() || url.starts_with('#') {
        return;
    }
    if image && url.starts_with("data:image/") {
        return;
    }
    let parsed = Url::parse(url)
        .or_else(|_| Url::parse("https://modrinth.com/").unwrap().join(url));
    if let Ok(parsed) = parsed
        && matches!(parsed.scheme(), "http" | "https" | "file")
    {
        targets.push(LinkTarget {
            field: "description".into(),
            url: parsed.to_string(),
            image,
        });
    }
}

pub(super) fn known_download(url: &Url, image: bool) -> bool {
    if image {
        return false;
    }
    let path = url.path().to_ascii_lowercase();
    let download_route = path.contains("/releases/download/")
        || path.contains("/archive/refs/")
        || (super::from_domains(url, &["github.com"])
            && path.contains("/raw/"))
        || (super::from_domains(url, &["drive.google.com"])
            && matches!(path.as_str(), "/uc" | "/download"))
        || (super::from_domains(url, &["curseforge.com"])
            && path.contains("/download"))
        || (super::from_domains(url, &["api.modrinth.com"])
            && path.ends_with("/download"));
    let download_query = url.query_pairs().any(|(key, value)| {
        matches!(key.as_ref(), "download" | "dl")
            && !matches!(value.as_ref(), "0" | "false")
    });
    download_route || download_query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rendered_links_and_images_without_code_or_duplicate_labels() {
        assert!(extract("example.com is sample text").is_empty());
        let targets = extract(
            r#"
[docs](https://docs.project.dev)
![preview](https://cdn.project.dev/image.png)
[![badge](https://badges.project.dev/status)](https://project.dev)
<a href="https://project.dev/?a=1&amp;b=2">link</a>
<img src="https://cdn.project.dev/other.png">
<video src="https://cdn.project.dev/movie.mp4" poster="https://cdn.project.dev/poster.png"></video>
<img srcset="https://cdn.project.dev/small.png 1x, https://cdn.project.dev/large.png 2x">
[reference][ref]

[ref]: https://reference.project.dev

https://plain.project.dev
~~~
https://ignored.project.dev
~~~
"#,
        );
        assert!(
            targets.iter().any(
                |target| target.image && target.url.ends_with("/image.png")
            )
        );
        assert!(
            targets
                .iter()
                .any(|target| target.url.ends_with("?a=1&b=2"))
        );
        assert!(
            targets
                .iter()
                .any(|target| target.url == "https://reference.project.dev/")
        );
        assert!(!targets.iter().any(|target| target.url.contains("ignored")));
        assert!(
            targets.iter().any(
                |target| target.image && target.url.ends_with("/large.png")
            )
        );
        assert!(
            targets
                .iter()
                .any(|target| !target.image
                    && target.url.ends_with("/movie.mp4"))
        );
        assert!(
            extract("\u{0060}https://ignored.project.dev\u{0060}").is_empty()
        );
    }

    #[test]
    fn download_routes_and_images() {
        assert!(known_download(
            &Url::parse(
                "https://github.com/owner/repo/releases/download/v1/file"
            )
            .unwrap(),
            false
        ));
        assert!(!known_download(
            &Url::parse("https://project.dev/manual.pdf").unwrap(),
            false
        ));
        assert!(!known_download(
            &Url::parse("https://project.dev/badge.svg").unwrap(),
            true
        ));
        assert_eq!(
            extract("[file](file:///tmp/file.jar)")[0].url,
            "file:///tmp/file.jar"
        );
    }
}
