use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::database::models::categories::Category;
use crate::models::disclosures::ProjectDisclosure;
use crate::models::projects::{Project, Version};

mod description;
mod disclosures;
mod gallery;
mod icon;
mod language;
mod license;
mod links;
mod moderation;
mod name;
mod permissions;
mod server_settings;
mod summary;
mod tags;
mod text;
mod versions;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    strum::EnumIter,
    utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectNagKind {
    // Project name
    ProjectNameSlur,
    ProjectNameProfanity,
    ProjectNameNonStandardText,
    ProjectNameVersion,
    MinecraftTitleClause,

    // Project summary
    ProjectSummarySlur,
    ProjectSummaryProfanity,
    ProjectSummaryNonStandardText,
    ProjectSummaryNonEnglish,
    ProjectSummaryMatchesTitle,
    SummaryTooShort,
    ProjectSummarySpam,
    SummarySpecialFormatting,
    ProjectSummaryLinks,

    // Project icon
    AddIcon,

    // Gallery
    GalleryTextSlur,
    GalleryTextProfanity,
    GalleryTextNonStandard,
    UploadGalleryImage,
    FeatureGalleryImage,

    // Project description
    ProjectDescriptionSlur,
    ProjectDescriptionProfanity,
    ProjectDescriptionNonStandardText,
    ProjectDescriptionNonEnglish,
    ProjectDescriptionMatchesSummary,
    AddDescription,
    DescriptionTooShort,
    ProjectDescriptionSpam,
    LongHeaders,
    DescriptionEndsWithHeader,
    AdjacentHeaders,
    MissingAltText,

    // License
    SelectLicense,
    AddCustomLicenseDetails,

    // External links
    AddLinks,
    AddLinksServer,
    LinkValidation,
    GplLicenseSourceRequired,

    // Permissions
    ReviewPermissions,

    // Server settings
    SelectCountry,
    AllLanguages,
    AddJavaAddress,
    SelectCompatibility,
    TooManyLanguages,
    SelectLanguage,

    // Tags
    SelectTags,
    TooManyTags,
    TooManyTagsServer,
    MultipleResolutionTags,
    AllTagsSelected,

    // Versions
    UploadVersion,
    SelectEnvironment,

    // Disclosures
    CheckDisclosures,
    DisclosuresSpecialFormatting,

    // Moderation
    ModeratorFeedback,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ProjectNagSeverity {
    Required,
    Warning,
    Suggestion,
}

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
pub struct ProjectNag {
    pub kind: ProjectNagKind,
    pub severity: ProjectNagSeverity,
    #[schema(value_type = serde_json::Value)]
    pub details: Value,
}

impl ProjectNag {
    pub(super) fn new(
        kind: ProjectNagKind,
        severity: ProjectNagSeverity,
    ) -> Self {
        Self {
            kind,
            severity,
            details: json!({}),
        }
    }

    pub(super) fn with_details(mut self, details: Value) -> Self {
        debug_assert!(details.is_object());
        self.details = details;
        self
    }
}

pub fn validate(project: &Project, versions: &[Version]) -> Vec<ProjectNag> {
    validate_inner(project, versions, None, None)
}

pub fn is_project_summary_non_english(project: &Project) -> bool {
    summary::is_non_english(project)
}

pub fn is_project_description_non_english(project: &Project) -> bool {
    description::is_non_english(project)
}

pub fn validate_with_context(
    project: &Project,
    versions: &[Version],
    available_categories: &[Category],
    disclosures: &[ProjectDisclosure],
) -> Vec<ProjectNag> {
    validate_inner(
        project,
        versions,
        Some(available_categories),
        Some(disclosures),
    )
}

fn validate_inner(
    project: &Project,
    versions: &[Version],
    available_categories: Option<&[Category]>,
    disclosures: Option<&[ProjectDisclosure]>,
) -> Vec<ProjectNag> {
    let mut nags = [
        name::validate,
        summary::validate,
        icon::validate,
        gallery::validate,
        description::validate,
        license::validate,
    ]
    .into_iter()
    .flat_map(|validate| validate(project))
    .collect::<Vec<_>>();

    nags.extend(links::validate(project, versions));
    nags.extend(permissions::validate(versions));
    nags.extend(versions::validate(project, versions));
    nags.extend(
        [server_settings::validate, moderation::validate]
            .into_iter()
            .flat_map(|validate| validate(project)),
    );
    nags.extend(tags::validate(project, available_categories));
    nags.extend(disclosures::validate(project, disclosures));
    nags
}

pub fn has_required_nags(project: &Project, versions: &[Version]) -> bool {
    validate(project, versions)
        .iter()
        .any(|nag| nag.severity == ProjectNagSeverity::Required)
}

#[derive(Clone, Copy, Default)]
pub struct LinkValidationScope {
    pub external: bool,
    pub license: bool,
    pub description: bool,
}

impl LinkValidationScope {
    fn includes(self, field: &str) -> bool {
        match field {
            "description" => self.description,
            "license" => self.license,
            _ => self.external,
        }
    }

    fn includes_nag(self, nag: &ProjectNag) -> bool {
        self.includes(nag.details["field"].as_str().unwrap_or_default())
            || ((self.external || self.license)
                && nag.details["reason"] == "duplicate")
    }
}

pub fn validate_link_fields(
    project: &Project,
    scope: LinkValidationScope,
) -> Vec<ProjectNag> {
    let mut nags = links::validate_static(project);
    nags.extend(license::validate_custom_details(project));
    nags.retain(|nag| scope.includes_nag(nag));
    nags
}

pub fn validate_link_input(
    links: &std::collections::HashMap<String, String>,
    license_id: &str,
    license_url: Option<&str>,
    description: &str,
) -> Vec<ProjectNag> {
    let mut nags = links::validate_input(links, license_url, description);
    nags.extend(license::validate_custom_license(license_id, license_url));
    nags
}
