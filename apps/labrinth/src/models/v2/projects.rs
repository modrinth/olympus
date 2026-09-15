use std::convert::TryFrom;

use std::collections::HashMap;

use super::super::ids::OrganizationId;
use crate::database::models::{
    DBProjectDisclosure, DBProjectId, DatabaseError, version_item,
};
use crate::models::disclosures::ProjectDisclosureType;
use crate::models::ids::{ProjectId, TeamId, ThreadId, VersionId};
use crate::models::projects::{
    Dependency, License, Link, Loader, ModeratorMessage, MonetizationStatus,
    Project, ProjectStatus, Version, VersionFile, VersionStatus, VersionType,
};
use crate::routes::v2_reroute::{self, capitalize_first};
use ariadne::ids::UserId;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use validator::Validate;
use xredis::RedisPool;

/// A project returned from the API
#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LegacyProject {
    /// Relevant V2 fields- these were removed or modified in V3,
    /// and are now part of the dynamic fields system
    /// The support range for the client project*
    pub client_side: LegacySideType,
    /// The support range for the server project
    pub server_side: LegacySideType,
    /// A list of game versions this project supports
    pub game_versions: Vec<String>,
    /// The environments this project supports
    pub environment: Vec<String>,

    // All other fields are the same as V3
    // If they change, or their constituent types change, we may need to
    // add a new struct for them here.
    pub id: ProjectId,
    pub slug: Option<String>,
    pub project_type: String,
    pub team: TeamId,
    pub organization: Option<OrganizationId>,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub requested_status: Option<ProjectStatus>,
    pub moderator_message: Option<ModeratorMessage>,
    pub license: License,
    pub downloads: u32,
    pub followers: u32,
    pub categories: Vec<String>,
    pub additional_categories: Vec<String>,
    pub loaders: Vec<String>,
    pub versions: Vec<VersionId>,
    pub icon_url: Option<String>,
    pub raw_icon_url: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<DonationLink>>,
    pub gallery: Vec<LegacyGalleryItem>,
    pub color: Option<u32>,
    pub thread_id: ThreadId,
    pub monetization_status: MonetizationStatus,
}

impl LegacyProject {
    // Returns visible v2 project_type and also 'og' selected project type
    // These are often identical, but we want to display 'mod' for datapacks and plugins
    // The latter can be used for further processing, such as determining side types of plugins
    pub fn get_project_type(project_types: &[String]) -> (String, String) {
        // V2 versions only have one project type- v3 versions can rarely have multiple.
        // We'll prioritize 'modpack' first, and if neither are found, use the first one.
        // If there are no project types, default to 'project'
        let mut project_types = project_types.to_vec();
        if project_types.contains(&"modpack".to_string()) {
            project_types = vec!["modpack".to_string()];
        }

        let og_project_type = project_types
            .first()
            .cloned()
            .unwrap_or("project".to_string()); // Default to 'project' if none are found

        let project_type =
            if og_project_type == "datapack" || og_project_type == "plugin" {
                // These are not supported in V2, so we'll just use 'mod' instead
                "mod".to_string()
            } else {
                og_project_type.clone()
            };

        (project_type, og_project_type)
    }

    // Convert from a standard V3 project to a V2 project
    // Requires any queried versions to be passed in, to get access to certain version fields contained within.
    // - This can be any version, because the fields are ones that used to be on the project itself.
    // - Its conceivable that certain V3 projects that have many different ones may not have the same fields on all of them.
    // It's safe to use a db version_item for this as the only info is side types, game versions, and loader fields (for loaders), which used to be public on project anyway.
    pub fn from(
        data: Project,
        versions_item: Option<version_item::VersionQueryResult>,
    ) -> Self {
        let mut client_side = LegacySideType::Unknown;
        let mut server_side = LegacySideType::Unknown;

        // V2 versions only have one project type- v3 versions can rarely have multiple.
        // We'll prioritize 'modpack' first, and if neither are found, use the first one.
        // If there are no project types, default to 'project'
        let project_types = data.project_types;
        let (mut project_type, og_project_type) =
            Self::get_project_type(&project_types);

        let mut loaders = data.loaders;

        let game_versions = data
            .fields
            .get("game_versions")
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| v.to_string())
            .collect();
        let environment = data
            .fields
            .get("environment")
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| v.to_string())
            .collect();

        if let Some(versions_item) = versions_item {
            // Extract side types from remaining fields
            let fields = versions_item
                .version_fields
                .iter()
                .map(|f| {
                    (f.field_name.clone(), f.value.clone().serialize_internal())
                })
                .collect::<HashMap<_, _>>();
            (client_side, server_side) =
                v2_reroute::convert_v3_side_types_to_v2_side_types(
                    &fields,
                    Some(&*og_project_type),
                );

            // - if loader is mrpack, this is a modpack
            // the loaders are whatever the corresponding loader fields are
            if loaders.contains(&"mrpack".to_string()) {
                project_type = "modpack".to_string();
                if let Some(mrpack_loaders) =
                    data.fields.iter().find(|f| f.0 == "mrpack_loaders")
                {
                    let values = mrpack_loaders
                        .1
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>();

                    // drop mrpack from loaders
                    loaders = loaders
                        .into_iter()
                        .filter(|l| l != "mrpack")
                        .collect::<Vec<_>>();
                    // and replace with mrpack_loaders
                    loaders.extend(values);
                    // remove duplicate loaders
                    loaders = loaders.into_iter().unique().collect::<Vec<_>>();
                }
            }
        }

        let issues_url = data.link_urls.get("issues").map(|l| l.url.clone());
        let source_url = data.link_urls.get("source").map(|l| l.url.clone());
        let wiki_url = data.link_urls.get("wiki").map(|l| l.url.clone());
        let discord_url = data.link_urls.get("discord").map(|l| l.url.clone());

        let donation_urls = data
            .link_urls
            .iter()
            .filter(|(_, l)| l.donation)
            .map(|(_, l)| DonationLink::try_from(l.clone()).ok())
            .collect::<Option<Vec<_>>>();

        Self {
            id: data.id,
            slug: data.slug,
            project_type,
            team: data.team_id,
            organization: data.organization,
            title: data.name,
            description: data.summary, // V2 description is V3 summary
            body: data.description,    // V2 body is V3 description
            body_url: None,            // Always None even in V2
            published: data.published,
            updated: data.updated,
            approved: data.approved,
            queued: data.queued,
            status: data.status,
            requested_status: data.requested_status,
            moderator_message: data.moderator_message,
            license: data.license,
            downloads: data.downloads,
            followers: data.followers,
            categories: data.categories,
            additional_categories: data.additional_categories,
            loaders,
            versions: data.versions,
            icon_url: data.icon_url,
            raw_icon_url: data.raw_icon_url,
            issues_url,
            source_url,
            wiki_url,
            discord_url,
            donation_urls,
            gallery: data
                .gallery
                .into_iter()
                .map(LegacyGalleryItem::from)
                .collect(),
            color: data.color,
            thread_id: data.thread_id,
            monetization_status: data.monetization_status,
            client_side,
            server_side,
            game_versions,
            environment,
        }
    }

    // Because from needs a version_item, this is a helper function to get many from one db query.
    pub async fn from_many(
        data: Vec<Project>,
        pool: &crate::database::PgPool,
        redis: &RedisPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let version_ids: Vec<_> = data
            .iter()
            .filter_map(|p| p.versions.first().map(|i| (*i).into()))
            .collect();
        let project_ids: Vec<DBProjectId> =
            data.iter().map(|p| p.id.into()).collect();

        let example_versions =
            version_item::DBVersion::get_many(&version_ids, pool, redis)
                .await?;
        let archived_disclosure_ids = DBProjectDisclosure::projects_with_type(
            ProjectDisclosureType::Archived,
            &project_ids,
            pool,
        )
        .await?;

        let mut legacy_projects = Vec::new();
        for project in data {
            let version_item = example_versions
                .iter()
                .find(|v| v.inner.project_id == project.id.into())
                .cloned();
            let has_archived_disclosure =
                archived_disclosure_ids.contains(&project.id.into());
            let mut project = LegacyProject::from(project, version_item);
            if has_archived_disclosure
                && project.status == ProjectStatus::Approved
            {
                project.status = ProjectStatus::Archived;
            }
            legacy_projects.push(project);
        }
        Ok(legacy_projects)
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Copy, utoipa::ToSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum LegacySideType {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

impl std::fmt::Display for LegacySideType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl LegacySideType {
    // These are constant, so this can remove unnecessary allocations (`to_string`)
    pub fn as_str(&self) -> &'static str {
        match self {
            LegacySideType::Required => "required",
            LegacySideType::Optional => "optional",
            LegacySideType::Unsupported => "unsupported",
            LegacySideType::Unknown => "unknown",
        }
    }

    pub fn from_string(string: &str) -> LegacySideType {
        match string {
            "required" => LegacySideType::Required,
            "optional" => LegacySideType::Optional,
            "unsupported" => LegacySideType::Unsupported,
            _ => LegacySideType::Unknown,
        }
    }
}

/// A specific version of a project
#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LegacyVersion {
    /// Relevant V2 fields- these were removed or modified in V3,
    /// and are now part of the dynamic fields system
    /// A list of game versions this project supports
    pub game_versions: Vec<String>,

    /// A list of loaders this project supports (has a newtype struct)
    pub loaders: Vec<Loader>,

    /// The environment this version supports
    pub environment: String,

    pub id: VersionId,
    pub project_id: ProjectId,
    pub author_id: UserId,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    pub changelog_url: Option<String>,
    pub date_published: DateTime<Utc>,
    pub downloads: u32,
    pub version_type: VersionType,
    pub status: VersionStatus,
    pub requested_status: Option<VersionStatus>,
    pub files: Vec<VersionFile>,
    pub dependencies: Vec<Dependency>,
}

impl From<Version> for LegacyVersion {
    fn from(data: Version) -> Self {
        let mut game_versions = Vec::new();
        if let Some(value) =
            data.fields.get("game_versions").and_then(|v| v.as_array())
        {
            for gv in value {
                if let Some(game_version) = gv.as_str() {
                    game_versions.push(game_version.to_string());
                }
            }
        }
        let environment = data
            .fields
            .get("environment")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown")
            .to_string();

        // - if loader is mrpack, this is a modpack
        // the v2 loaders are whatever the corresponding loader fields are
        let mut loaders =
            data.loaders.into_iter().map(|l| l.0).collect::<Vec<_>>();
        if loaders.contains(&"mrpack".to_string())
            && let Some((_, mrpack_loaders)) = data
                .fields
                .into_iter()
                .find(|(key, _)| key == "mrpack_loaders")
            && let Ok(mrpack_loaders) = serde_json::from_value(mrpack_loaders)
        {
            loaders = mrpack_loaders;
        }
        let loaders = loaders.into_iter().map(Loader).collect::<Vec<_>>();

        Self {
            id: data.id,
            project_id: data.project_id,
            author_id: data.author_id,
            featured: data.featured,
            name: data.name,
            version_number: data.version_number,
            changelog: data.changelog,
            changelog_url: None, // Always None even in V2
            date_published: data.date_published,
            downloads: data.downloads,
            version_type: data.version_type,
            status: data.status,
            requested_status: data.requested_status,
            files: data.files,
            dependencies: data.dependencies,
            game_versions,
            loaders,
            environment,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct LegacyGalleryItem {
    pub url: String,
    pub raw_url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: i64,
}

impl LegacyGalleryItem {
    fn from(data: crate::models::projects::GalleryItem) -> Self {
        Self {
            url: data.url,
            raw_url: data.raw_url,
            featured: data.featured,
            title: data.name,
            description: data.description,
            created: data.created,
            ordering: data.ordering,
        }
    }
}

#[derive(
    Serialize, Deserialize, Validate, Clone, Eq, PartialEq, utoipa::ToSchema,
)]
pub struct DonationLink {
    pub id: String,
    pub platform: String,
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub url: String,
}

impl TryFrom<Link> for DonationLink {
    type Error = String;
    fn try_from(link: Link) -> Result<Self, String> {
        if !link.donation {
            return Err("Not a donation".to_string());
        }
        Ok(Self {
            platform: capitalize_first(&link.platform),
            url: link.url,
            id: link.platform,
        })
    }
}

pub fn validate_donation_platforms(
    links: &[DonationLink],
) -> Result<(), crate::routes::ApiError> {
    let mut platforms = std::collections::HashSet::new();
    for link in links {
        if !matches!(
            link.id.as_str(),
            "patreon" | "bmac" | "paypal" | "github" | "ko-fi" | "other"
        ) {
            return Err(crate::routes::ApiError::Request(eyre::eyre!(
                "each donation link must specify a donation platform"
            )));
        }
        if !platforms.insert(&link.id) {
            return Err(crate::routes::ApiError::Request(eyre::eyre!(
                "donation platforms must not be repeated"
            )));
        }
    }
    Ok(())
}
