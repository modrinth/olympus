mod markdown;

use serde_json::json;

use self::markdown::DescriptionMarkdown;
use super::text::{
    ProfanityKind, contains_description_spam, extract_description_blocks,
    extract_description_text, has_image_without_alt_text,
    has_sufficient_english_blocks, is_confidently_non_english_short_text,
    js_string_length, non_standard_text_ratio, normalize_project_field_text,
    profanity_matches, project_requires_english, project_text_similarity,
};
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

use crate::models::projects::Project;

const MIN_DESCRIPTION_CHARS: usize = 125;
const MIN_DESCRIPTION_LANGUAGE_CHARS: usize = 35;
const MAX_DESCRIPTION_SUMMARY_SIMILARITY: f64 = 0.8;
const MAX_PROFANITY_COUNT: usize = 2;
const NON_STANDARD_TEXT_FAILURE_THRESHOLD: f64 = 0.05;

pub(super) fn validate(project: &Project) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let description = project.description.as_str();
    let markdown = DescriptionMarkdown::parse(description);
    let description_without_code = markdown.without_code();
    let normalized_description = normalize_project_field_text(description);
    let text = extract_description_text(&description_without_code);
    let has_spam = has_description_spam(&description_without_code);
    let normalized_text =
        normalize_project_field_text(&description_without_code);
    let normalized_text = extract_description_text(&normalized_text);
    let blocks = extract_description_blocks(&description_without_code);
    let profanity = profanity_matches(description);

    if let Some(matched) = profanity
        .iter()
        .find(|matched| matched.kind == ProfanityKind::Slur)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectDescriptionSlur,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if let Some(matched) = profanity
        .iter()
        .filter(|matched| matched.kind == ProfanityKind::Profanity)
        .nth(MAX_PROFANITY_COUNT)
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::ProjectDescriptionProfanity,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "value": matched.raw_text })),
        );
    }
    if non_standard_text_ratio(&description_without_code)
        >= NON_STANDARD_TEXT_FAILURE_THRESHOLD
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionNonStandardText,
            ProjectNagSeverity::Required,
        ));
    }
    if !has_spam && is_non_english_text(project, &text, &blocks) {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionNonEnglish,
            ProjectNagSeverity::Required,
        ));
    }
    if normalized_description.is_empty() {
        nags.push(ProjectNag::new(
            ProjectNagKind::AddDescription,
            ProjectNagSeverity::Required,
        ));
    } else {
        let description_length = js_string_length(&normalized_text);
        if description_length < MIN_DESCRIPTION_CHARS {
            nags.push(
                ProjectNag::new(
                    ProjectNagKind::DescriptionTooShort,
                    ProjectNagSeverity::Required,
                )
                .with_details(json!({
                    "length": description_length,
                    "min_chars": MIN_DESCRIPTION_CHARS,
                })),
            );
        }
    }
    if project_text_similarity(&normalized_text, &project.summary)
        >= MAX_DESCRIPTION_SUMMARY_SIMILARITY
    {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionMatchesSummary,
            ProjectNagSeverity::Required,
        ));
    }
    if has_spam {
        nags.push(ProjectNag::new(
            ProjectNagKind::ProjectDescriptionSpam,
            ProjectNagSeverity::Required,
        ));
    }
    let long_headers = markdown.long_header_count();
    if long_headers > 0 {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::LongHeaders,
                ProjectNagSeverity::Required,
            )
            .with_details(json!({ "count": long_headers })),
        );
    }
    if markdown.ends_with_header() {
        nags.push(ProjectNag::new(
            ProjectNagKind::DescriptionEndsWithHeader,
            ProjectNagSeverity::Required,
        ));
    }
    if markdown.has_adjacent_same_level_headers() {
        nags.push(ProjectNag::new(
            ProjectNagKind::AdjacentHeaders,
            ProjectNagSeverity::Required,
        ));
    }
    if has_image_without_alt_text(&description_without_code) {
        nags.push(ProjectNag::new(
            ProjectNagKind::MissingAltText,
            ProjectNagSeverity::Warning,
        ));
    }

    nags
}

pub(super) fn is_non_english(project: &Project) -> bool {
    let markdown = DescriptionMarkdown::parse(&project.description);
    let description_without_code = markdown.without_code();
    let text = extract_description_text(&description_without_code);
    let blocks = extract_description_blocks(&description_without_code);
    is_non_english_text(project, &text, &blocks)
}

fn is_non_english_text(
    project: &Project,
    text: &str,
    blocks: &[String],
) -> bool {
    let length = js_string_length(text);
    project_requires_english(project)
        && length >= MIN_DESCRIPTION_LANGUAGE_CHARS
        && (length >= MIN_DESCRIPTION_CHARS
            || is_confidently_non_english_short_text(&blocks.join(" ")))
        && !has_sufficient_english_blocks(blocks)
}

fn has_description_spam(markdown: &str) -> bool {
    contains_description_spam(markdown)
}

#[cfg(test)]
mod tests {
    use super::has_description_spam;

    #[test]
    fn description_spam_ignores_formatting() {
        assert!(!has_description_spam("======== ━━━━━━━━ !!!!!!!!"));
        assert!(has_description_spam(&"a".repeat(64)));
    }

    #[test]
    fn description_spam_allows_isolated_repetition() {
        assert!(!has_description_spam(
            "aaaaaaaa This project provides configurable tools for exploring worlds, organizing resources, managing settings, sharing creations, and helping players understand every available feature through clear documentation."
        ));
    }
}
