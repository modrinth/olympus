use super::text::has_paired_html_formatting;
use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::disclosures::ProjectDisclosure;
use crate::models::v2::projects::LegacyProject;

pub(super) fn validate(
    project: &crate::models::projects::Project,
    disclosures: Option<&[ProjectDisclosure]>,
) -> Vec<ProjectNag> {
    let mut nags = Vec::new();
    if disclosures.is_some_and(|disclosures| disclosures.is_empty()) {
        let (project_type, _) =
            LegacyProject::get_project_type(&project.project_types);
        nags.push(
            ProjectNag::new(
                ProjectNagKind::CheckDisclosures,
                ProjectNagSeverity::Suggestion,
            )
            .with_details(serde_json::json!({ "project_type": project_type })),
        );
    }

    if disclosures.is_some_and(|disclosures| {
        disclosures.iter().any(disclosure_has_paired_html)
    }) {
        nags.push(ProjectNag::new(
            ProjectNagKind::DisclosuresSpecialFormatting,
            ProjectNagSeverity::Required,
        ));
    }

    nags
}

fn disclosure_has_paired_html(disclosure: &ProjectDisclosure) -> bool {
    match disclosure {
        ProjectDisclosure::AiContent { note, .. }
        | ProjectDisclosure::AiFunctionality { note, .. }
        | ProjectDisclosure::Advertisements { note }
        | ProjectDisclosure::EpilepsyTriggers { note }
        | ProjectDisclosure::SystemInteractions { note, .. }
        | ProjectDisclosure::Archived { note } => {
            note.as_deref().is_some_and(has_paired_html_formatting)
        }
        ProjectDisclosure::Telemetry { data_collected, .. } => data_collected
            .iter()
            .any(|text| has_paired_html_formatting(text)),
        ProjectDisclosure::DerivativeWork { sources } => {
            sources.iter().any(|source| {
                has_paired_html_formatting(&source.label)
                    || source
                        .note
                        .as_deref()
                        .is_some_and(has_paired_html_formatting)
            })
        }
        ProjectDisclosure::PaidFeatures { features } => {
            features.iter().any(|text| has_paired_html_formatting(text))
        }
    }
}
