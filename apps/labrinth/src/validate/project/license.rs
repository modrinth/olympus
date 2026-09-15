use super::{ProjectNag, ProjectNagKind, ProjectNagSeverity};
use crate::models::v2::projects::LegacyProject;

pub(super) fn validate(
    project: &crate::models::projects::Project,
) -> Vec<super::ProjectNag> {
    let mut nags = Vec::new();
    let is_minecraft_server = project.components.minecraft_server.is_some();
    let (project_type, _) =
        LegacyProject::get_project_type(&project.project_types);
    let license = &project.license;
    let has_unknown_license = matches!(
        license.id.as_str(),
        "LicenseRef-Unknown" | "NOASSERTION" | "LicenseRef-NOASSERTION"
    );

    if has_unknown_license && !is_minecraft_server {
        nags.push(
            ProjectNag::new(
                ProjectNagKind::SelectLicense,
                ProjectNagSeverity::Required,
            )
            .with_details(serde_json::json!({ "project_type": project_type })),
        );
    }

    nags.extend(validate_custom_details(project));
    nags
}

pub(super) fn validate_custom_details(
    project: &crate::models::projects::Project,
) -> Vec<ProjectNag> {
    let license = &project.license;
    validate_custom_license(&license.id, license.url.as_deref())
}

pub(super) fn validate_custom_license(
    id: &str,
    url: Option<&str>,
) -> Vec<ProjectNag> {
    const BUILTIN_LICENSES: &[&str] = &[
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "CC0-1.0",
        "CC-BY-4.0",
        "CC-BY-SA-4.0",
        "CC-BY-NC-4.0",
        "CC-BY-NC-SA-4.0",
        "CC-BY-ND-4.0",
        "CC-BY-NC-ND-4.0",
        "AGPL-3.0",
        "LGPL-2.1",
        "LGPL-3.0",
        "GPL-2.0",
        "GPL-3.0",
        "ISC",
        "MIT",
        "MPL-2.0",
        "Zlib",
    ];
    let base_id = id.trim_end_matches("-only").trim_end_matches("-or-later");
    let custom = !BUILTIN_LICENSES.contains(&base_id)
        && !matches!(
            id,
            "LicenseRef-Unknown"
                | "LicenseRef-All-Rights-Reserved"
                | "LicenseRef-NOASSERTION"
                | "NOASSERTION"
                | "arr"
        );
    let missing_name = id.trim().is_empty()
        || id
            .strip_prefix("LicenseRef-")
            .is_some_and(|name| name.trim_matches([' ', '-']).is_empty());
    let missing_url = url.is_none_or(|url| url.trim().is_empty());
    if custom && (missing_name || missing_url) {
        vec![ProjectNag::new(ProjectNagKind::AddCustomLicenseDetails, ProjectNagSeverity::Required)
			.with_details(serde_json::json!({ "field": "license", "missing_name": missing_name, "missing_url": missing_url }))]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::validate_custom_license;

    #[test]
    fn custom_licenses_require_both_details() {
        for (id, url) in [
            ("LicenseRef-", Some("https://license.project.dev")),
            ("LicenseRef-Custom", None),
            ("Unlicense", None),
            ("", Some("https://license.project.dev")),
        ] {
            assert_eq!(validate_custom_license(id, url).len(), 1, "{id}");
        }
        for id in ["MIT", "GPL-3.0-or-later", "LicenseRef-All-Rights-Reserved"]
        {
            assert!(validate_custom_license(id, None).is_empty());
        }
        assert!(
            validate_custom_license(
                "LicenseRef-Custom",
                Some("https://license.project.dev")
            )
            .is_empty()
        );
    }
}
