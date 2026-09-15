use std::collections::BTreeSet;

use crate::{
    database::models::categories::Category,
    models::{projects::Project, v2::projects::LegacyProject},
};

use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};

const MAX_TAG_COUNT: usize = 8;
const MAX_TAG_COUNT_SERVER: usize = 18;
const RESOLUTION_TAGS: [&str; 8] =
    ["8x-", "16x", "32x", "48x", "64x", "128x", "256x", "512x+"];

pub(super) fn validate(
    project: &Project,
    available_categories: Option<&[Category]>,
) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    let tag_count =
        project.categories.len() + project.additional_categories.len();
    let is_minecraft_server = project.components.minecraft_server.is_some();
    let project_type =
        LegacyProject::get_project_type(&project.project_types).0;

    if !project.versions.is_empty() && project.categories.is_empty() {
        nags.push(ProjectNag::new(
            ProjectNagKind::SelectTags,
            ProjectNagSeverity::Suggestion,
        ));
    }

    if !is_minecraft_server && tag_count > MAX_TAG_COUNT {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyTags,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "tag_count": tag_count,
                "max_tag_count": MAX_TAG_COUNT,
            })),
        );
    }

    if is_minecraft_server && tag_count > MAX_TAG_COUNT_SERVER {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::TooManyTagsServer,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({
                "tag_count": tag_count,
                "max_tag_count": MAX_TAG_COUNT_SERVER,
            })),
        );
    }

    let mut resolution_tags = project
        .categories
        .iter()
        .chain(&project.additional_categories)
        .filter(|tag| RESOLUTION_TAGS.contains(&tag.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    resolution_tags.sort_by_key(|tag| {
        RESOLUTION_TAGS
            .iter()
            .position(|resolution| resolution == tag)
            .unwrap_or(RESOLUTION_TAGS.len())
    });
    if project_type == "resourcepack" && resolution_tags.len() > 1 {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::MultipleResolutionTags,
                ProjectNagSeverity::Warning,
            )
            .with_details(serde_json::json!({
                "count": resolution_tags.len(),
                "tags": resolution_tags.join("|"),
            })),
        );
    }

    if let Some(available_categories) = available_categories
        && let Some(total_available_tags) = all_available_tags_selected(
            project
                .categories
                .iter()
                .chain(&project.additional_categories)
                .map(String::as_str),
            available_categories
                .iter()
                .filter(|category| category.project_type == project_type)
                .map(|category| category.category.as_str()),
        )
    {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::AllTagsSelected,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({
                "total_available_tags": total_available_tags,
            })),
        );
    }

    nags
}

fn all_available_tags_selected<'a, 'b>(
    selected_tags: impl Iterator<Item = &'a str>,
    available_tags: impl Iterator<Item = &'b str>,
) -> Option<usize> {
    let selected_tags = selected_tags.collect::<BTreeSet<_>>();
    let available_tags = available_tags.collect::<BTreeSet<_>>();

    (!available_tags.is_empty() && available_tags.is_subset(&selected_tags))
        .then_some(available_tags.len())
}

#[cfg(test)]
mod tests {
    use super::all_available_tags_selected;

    #[test]
    fn all_available_tags_are_compared_by_value() {
        assert_eq!(
            all_available_tags_selected(
                ["combat", "magic", "mobs"].into_iter(),
                ["combat", "magic", "mobs"].into_iter(),
            ),
            Some(3)
        );
        assert_eq!(
            all_available_tags_selected(
                ["combat", "magic", "modpack-exclusive"].into_iter(),
                ["combat", "magic", "mobs"].into_iter(),
            ),
            None
        );
        assert_eq!(
            all_available_tags_selected(
                ["combat", "magic", "mobs", "modpack-exclusive"].into_iter(),
                ["combat", "magic", "mobs"].into_iter(),
            ),
            Some(3)
        );
        assert_eq!(
            all_available_tags_selected(
                ["modpack-exclusive"].into_iter(),
                std::iter::empty(),
            ),
            None
        );
    }
}
