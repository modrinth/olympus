use std::ops::Range;
use std::sync::LazyLock;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

static HTML_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<h[1-6]\b[^>]*>(.*?)</h[1-6]>").unwrap()
});
static ADJACENT_HTML_HEADERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)</h([1-6])>\s*<h([1-6])\b").unwrap());
static TRAILING_HTML_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)</h[1-6]>\s*(?:</[a-z][^>]*>\s*)*$").unwrap()
});

#[derive(Debug)]
struct MarkdownHeading {
    level: HeadingLevel,
    text: String,
    range: Range<usize>,
}

pub(super) struct DescriptionMarkdown<'a> {
    markdown: &'a str,
    code_ranges: Vec<Range<usize>>,
    headings: Vec<MarkdownHeading>,
}

impl<'a> DescriptionMarkdown<'a> {
    pub(super) fn parse(markdown: &'a str) -> Self {
        let mut code_ranges = Vec::new();
        let mut code_block_start = None;
        let mut headings = Vec::new();
        let mut current_heading = None;

        for (event, range) in Parser::new(markdown).into_offset_iter() {
            match event {
                Event::Start(Tag::CodeBlock(_)) => {
                    code_block_start = Some(range.start);
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(start) = code_block_start.take() {
                        code_ranges.push(start..range.end);
                    }
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    current_heading = Some(MarkdownHeading {
                        level,
                        text: String::new(),
                        range: range.start..range.end,
                    });
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(mut heading) = current_heading.take() {
                        heading.range.end = range.end;
                        headings.push(heading);
                    }
                }
                Event::Code(text) => {
                    if code_block_start.is_none() {
                        code_ranges.push(range);
                    }
                    if let Some(heading) = &mut current_heading {
                        heading.text.push_str(&text);
                    }
                }
                Event::Text(text)
                | Event::InlineMath(text)
                | Event::DisplayMath(text)
                | Event::FootnoteReference(text) => {
                    if let Some(heading) = &mut current_heading {
                        heading.text.push_str(&text);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some(heading) = &mut current_heading {
                        heading.text.push(' ');
                    }
                }
                _ => {}
            }
        }

        if let Some(start) = code_block_start {
            code_ranges.push(start..markdown.len());
        }

        Self {
            markdown,
            code_ranges,
            headings,
        }
    }

    pub(super) fn without_code(&self) -> String {
        self.replace_code(" ")
    }

    pub(super) fn long_header_count(&self) -> usize {
        let markdown_headers = self
            .headings
            .iter()
            .filter(|heading| header_is_long(&heading.text))
            .count();
        let without_code = self.replace_code(" ");
        let html_headers = HTML_HEADER
            .captures_iter(&without_code)
            .filter(|captures| header_is_long(&captures[1]))
            .count();

        markdown_headers + html_headers
    }

    pub(super) fn ends_with_header(&self) -> bool {
        let ends_with_markdown_header =
            self.headings.last().is_some_and(|heading| {
                self.markdown[heading.range.end..].trim().is_empty()
            });
        let without_code = self.replace_code("\n[code]\n");

        ends_with_markdown_header
            || TRAILING_HTML_HEADER.is_match(without_code.trim_end())
    }

    pub(super) fn has_adjacent_same_level_headers(&self) -> bool {
        let has_adjacent_markdown_headers =
            self.headings.windows(2).any(|headings| {
                let [previous, current] = headings else {
                    return false;
                };
                previous.level == current.level
                    && self.markdown[previous.range.end..current.range.start]
                        .trim()
                        .is_empty()
            });
        let without_code = self.replace_code("\n[code]\n");
        let has_adjacent_html_headers = ADJACENT_HTML_HEADERS
            .captures_iter(&without_code)
            .any(|captures| {
                captures.get(1).map(|level| level.as_str())
                    == captures.get(2).map(|level| level.as_str())
            });

        has_adjacent_markdown_headers || has_adjacent_html_headers
    }

    fn replace_code(&self, replacement: &str) -> String {
        if self.code_ranges.is_empty() {
            return self.markdown.to_owned();
        }

        let mut without_code = String::with_capacity(self.markdown.len());
        let mut previous_end = 0;
        for range in &self.code_ranges {
            without_code.push_str(&self.markdown[previous_end..range.start]);
            without_code.push_str(replacement);
            previous_end = range.end;
        }
        without_code.push_str(&self.markdown[previous_end..]);
        without_code
    }
}

fn header_is_long(header: &str) -> bool {
    let mut rendered = String::new();
    for event in Parser::new(header) {
        match event {
            Event::Text(text)
            | Event::Code(text)
            | Event::InlineMath(text)
            | Event::DisplayMath(text)
            | Event::FootnoteReference(text) => rendered.push_str(&text),
            _ => {}
        }
    }

    rendered.graphemes(true).count() > 80
}

#[cfg(test)]
mod tests {
    use super::DescriptionMarkdown;

    #[test]
    fn description_content_ignores_markdown_code() {
        let markdown = r#"
```yaml
# This is a YAML comment rather than a heading
homepage: https://bit.ly/example
image: "![](/missing-alt.png)"
```

`https://bit.ly/inline`
"#;

        let markdown = DescriptionMarkdown::parse(markdown);
        let without_code = markdown.without_code();
        assert!(!without_code.contains("YAML comment"));
        assert!(!without_code.contains("bit.ly"));
    }

    #[test]
    fn heading_checks_ignore_fenced_and_indented_code() {
        let long_comment = format!("# {}", "comment ".repeat(12));
        for markdown in [
            format!("```yaml\n{long_comment}\n```"),
            format!("~~~yaml\n{long_comment}\n~~~"),
            format!("````yaml\n```\n{long_comment}\n````"),
            format!("Example:\n\n```yaml\n{long_comment}\n```"),
            format!("```yaml\n{long_comment}"),
            format!("    {long_comment}"),
        ] {
            let markdown = DescriptionMarkdown::parse(&markdown);
            assert_eq!(markdown.long_header_count(), 0);
            assert!(!markdown.ends_with_header());
        }
    }

    #[test]
    fn code_blocks_separate_headers() {
        let markdown = DescriptionMarkdown::parse(
            "# First\n\n```yaml\n# comment\n```\n\n# Second",
        );

        assert!(!markdown.has_adjacent_same_level_headers());
        assert!(markdown.ends_with_header());
    }

    #[test]
    fn rendered_headers_are_still_validated() {
        let long_header = format!("# {}", "heading ".repeat(12));

        assert_eq!(
            DescriptionMarkdown::parse(&long_header).long_header_count(),
            1
        );
        assert!(DescriptionMarkdown::parse("Title\n=====").ends_with_header());
        assert!(
            DescriptionMarkdown::parse("## First\n\n## Second")
                .has_adjacent_same_level_headers()
        );
    }

    #[test]
    fn all_heading_levels_are_validated() {
        for level in 1..=6 {
            let prefix = "#".repeat(level);
            let long_text = "heading ".repeat(12);
            for (long_header, adjacent_headers, separated_headers) in [
                (
                    format!("{prefix} {long_text}"),
                    format!("{prefix} First\n\n{prefix} Second"),
                    format!(
                        "{prefix} First\n\n```yaml\n# comment\n```\n\n{prefix} Second"
                    ),
                ),
                (
                    format!("<h{level}>{long_text}</h{level}>"),
                    format!(
                        "<h{level}>First</h{level}>\n\n<h{level}>Second</h{level}>"
                    ),
                    format!(
                        "<h{level}>First</h{level}>\n\n```yaml\n# comment\n```\n\n<h{level}>Second</h{level}>"
                    ),
                ),
            ] {
                let markdown = DescriptionMarkdown::parse(&long_header);
                assert_eq!(markdown.long_header_count(), 1, "{long_header}");
                assert!(markdown.ends_with_header(), "{long_header}");
                assert!(
                    DescriptionMarkdown::parse(&adjacent_headers)
                        .has_adjacent_same_level_headers(),
                    "{adjacent_headers}"
                );
                assert!(
                    !DescriptionMarkdown::parse(&separated_headers)
                        .has_adjacent_same_level_headers(),
                    "{separated_headers}"
                );
            }
        }
    }
}
