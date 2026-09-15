use crate::database::PgPool;
use crate::database::models::version_item;
use crate::file_hosting::FileHost;
use crate::models;
use crate::models::ids::ImageId;
use crate::models::projects::{Loader, Project, ProjectStatus};
use crate::models::v2::projects::{
    DonationLink, LegacyProject, LegacySideType,
};
use crate::queue::session::AuthQueue;
use crate::routes::v3::project_creation::default_project_type;
use crate::routes::v3::project_creation::{CreateError, NewGalleryItem};
use crate::routes::{v2_reroute, v3};
use crate::search::SearchState;
use crate::util::kafka::KafkaClientState;
use actix_multipart::Multipart;
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use xredis::RedisPool;

use std::collections::HashMap;
use validator::Validate;

use super::version_creation::InitialVersionData;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(project_create);
}

pub fn default_requested_status() -> ProjectStatus {
    ProjectStatus::Approved
}

#[derive(Serialize, Deserialize, Validate, Clone)]
struct ProjectCreateData {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    #[serde(alias = "mod_name")]
    /// The title or name of the project.
    pub title: String,
    #[validate(length(min = 1, max = 64))]
    #[serde(default = "default_project_type")]
    /// The project type of this mod
    pub project_type: String,
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    #[serde(alias = "mod_slug")]
    /// The slug of a project, used for vanity URLs
    pub slug: String,
    #[validate(length(min = 3, max = 255))]
    #[serde(alias = "mod_description")]
    /// A short description of the project.
    pub description: String,
    #[validate(length(max = 65536))]
    #[serde(alias = "mod_body")]
    /// A long description of the project, in markdown.
    pub body: String,

    /// The support range for the client project
    pub client_side: LegacySideType,
    /// The support range for the server project
    pub server_side: LegacySideType,

    #[validate(nested, length(max = 32))]
    /// A list of initial versions to upload with the created project
    pub initial_versions: Vec<InitialVersionData>,
    #[validate(length(max = 3))]
    /// A list of the categories that the project is in.
    pub categories: Vec<String>,
    #[validate(length(max = 256))]
    #[serde(default = "Vec::new")]
    /// A list of the categories that the project is in.
    pub additional_categories: Vec<String>,

    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    /// An optional link to where to submit bugs or issues with the project.
    pub issues_url: Option<String>,
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    /// An optional link to the source code for the project.
    pub source_url: Option<String>,
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    /// An optional link to the project's wiki page or other relevant information.
    pub wiki_url: Option<String>,
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    /// An optional link to the project's license page
    pub license_url: Option<String>,
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    /// An optional link to the project's discord.
    pub discord_url: Option<String>,
    /// An optional list of all donation links the project has\
    #[validate(nested)]
    pub donation_urls: Option<Vec<DonationLink>>,

    /// An optional boolean. If true, the project will be created as a draft.
    pub is_draft: Option<bool>,

    /// The license id that the project follows
    pub license_id: String,

    #[validate(nested, length(max = 64))]
    /// The multipart names of the gallery items to upload
    pub gallery_items: Option<Vec<NewGalleryItem>>,
    #[serde(default = "default_requested_status")]
    /// The status of the mod to be set once it is approved
    pub requested_status: ProjectStatus,

    // Associations to uploaded images in body/description
    #[validate(length(max = 10))]
    #[serde(default)]
    pub uploaded_images: Vec<ImageId>,

    /// The id of the organization to create the project in
    pub organization_id: Option<models::ids::OrganizationId>,
}

/// Create a new project with initial versions.
#[utoipa::path(
	tag = "project creation",
    post,
    operation_id = "createProject",
    request_body(
        content(("multipart/form-data")),
        description = "Multipart payload containing `data` and uploaded files"
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = LegacyProject),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_CREATE"]))
)]
#[post("/project")]
pub async fn project_create(
    req: HttpRequest,
    payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    session_queue: Data<AuthQueue>,
    kafka_client: Data<KafkaClientState>,
    search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
    // Convert V2 multipart payload to V3 multipart payload
    let payload = v2_reroute::alter_actix_multipart(
        payload,
        req.headers().clone(),
        |legacy_create: ProjectCreateData, _| async move {
            // Side types will be applied to each version
            let client_side = legacy_create.client_side;
            let server_side = legacy_create.server_side;

            let project_type = legacy_create.project_type;

            let initial_versions = legacy_create
                .initial_versions
                .into_iter()
                .map(|v| {
                    let mut fields = HashMap::new();
                    fields.extend(
                        v2_reroute::convert_v2_side_types_to_v3_side_types(
                            client_side,
                            server_side,
                        ),
                    );
                    if let Some(environment) = v.environment {
                        fields.insert(
                            "environment".to_string(),
                            json!(environment),
                        );
                    }
                    fields.insert(
                        "game_versions".to_string(),
                        json!(v.game_versions),
                    );

                    // Modpacks now use the "mrpack" loader, and loaders are converted to loader fields.
                    // Setting of 'project_type' directly is removed, it's loader-based now.
                    if project_type == "modpack" {
                        fields.insert(
                            "mrpack_loaders".to_string(),
                            json!(v.loaders),
                        );
                    }

                    let loaders = if project_type == "modpack" {
                        vec![Loader("mrpack".to_string())]
                    } else {
                        v.loaders
                    };

                    v3::version_creation::InitialVersionData {
                        project_id: v.project_id,
                        file_parts: v.file_parts,
                        version_number: v.version_number,
                        version_title: v.version_title,
                        version_body: v.version_body,
                        dependencies: v.dependencies,
                        release_channel: v.release_channel,
                        loaders,
                        featured: v.featured,
                        primary_file: v.primary_file,
                        status: v.status,
                        file_types: v.file_types,
                        uploaded_images: v.uploaded_images,
                        ordering: v.ordering,
                        fields,
                    }
                })
                .collect();

            let mut link_urls = HashMap::new();
            if let Some(issue_url) = legacy_create.issues_url {
                link_urls.insert("issues".to_string(), issue_url);
            }
            if let Some(source_url) = legacy_create.source_url {
                link_urls.insert("source".to_string(), source_url);
            }
            if let Some(wiki_url) = legacy_create.wiki_url {
                link_urls.insert("wiki".to_string(), wiki_url);
            }
            if let Some(discord_url) = legacy_create.discord_url {
                link_urls.insert("discord".to_string(), discord_url);
            }
            if let Some(donation_urls) = legacy_create.donation_urls {
                crate::models::v2::projects::validate_donation_platforms(
                    &donation_urls,
                )
                .map_err(|error| {
                    CreateError::InvalidInput(error.to_string())
                })?;
                for donation_url in donation_urls {
                    link_urls.insert(donation_url.id, donation_url.url);
                }
            }

            Ok(v3::project_creation::ProjectCreateData {
                name: legacy_create.title,
                slug: legacy_create.slug,
                summary: legacy_create.description, // Description becomes summary
                description: legacy_create.body,    // Body becomes description
                initial_versions,
                categories: legacy_create.categories,
                additional_categories: legacy_create.additional_categories,
                license_url: legacy_create.license_url,
                link_urls,
                is_draft: legacy_create.is_draft,
                license_id: legacy_create.license_id,
                gallery_items: legacy_create.gallery_items,
                requested_status: legacy_create.requested_status,
                uploaded_images: legacy_create.uploaded_images,
                organization_id: legacy_create.organization_id,
            })
        },
    )
    .await?;

    // Call V3 project creation
    let response = v3::project_creation::project_create_internal(
        req,
        payload,
        client.clone(),
        redis.clone(),
        file_host,
        session_queue,
        kafka_client,
        search_state,
    )
    .await?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Project>(response).await {
        Ok(project) => {
            let version_item = match project.versions.first() {
                Some(vid) => {
                    version_item::DBVersion::get(
                        (*vid).into(),
                        &**client,
                        &redis,
                    )
                    .await?
                }
                None => None,
            };
            let project = LegacyProject::from(project, version_item);
            Ok(HttpResponse::Ok().json(project))
        }
        Err(response) => Ok(response),
    }
}
