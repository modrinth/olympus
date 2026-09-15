use super::version_creation::{InitialVersionData, try_create_version_fields};
use crate::auth::{
    AuthenticationError, get_user_from_headers, require_verified_email,
};
use crate::database::PgPool;
use crate::database::PgTransaction;
use crate::database::models::loader_fields::{
    Loader, LoaderField, LoaderFieldEnumValue,
};
use crate::database::models::thread_item::ThreadBuilder;
use crate::database::models::{self, DBUser, image_item};
use crate::file_hosting::{FileHost, FileHostPublicity, FileHostingError};
use crate::models::error::ApiError;
use crate::models::exp;
use crate::models::ids::{ImageId, OrganizationId, ProjectId, VersionId};
use crate::models::images::{Image, ImageContext};
use crate::models::pats::Scopes;
use crate::models::projects::{
    License, Link, MonetizationStatus, Project, ProjectStatus,
    SideTypesMigrationReviewStatus, VersionStatus,
};
use crate::models::teams::{OrganizationPermissions, ProjectPermissions};
use crate::models::threads::ThreadType;
use crate::models::v3::user_limits::UserLimits;
use crate::queue::session::AuthQueue;
use crate::search::SearchState;
use crate::util::guards::admin_key_guard;
use crate::util::img::upload_image_optimized;
use crate::util::kafka::KafkaClientState;
use crate::util::routes::read_from_field;
use crate::util::validate::validation_errors_to_string;
use actix_multipart::{Field, Multipart};
use actix_web::http::StatusCode;
use actix_web::web::{self, Data};
use actix_web::{HttpRequest, HttpResponse, post};
use ariadne::ids::UserId;
use ariadne::ids::base62_impl::to_base62;
use chrono::Utc;
use futures::stream::StreamExt;
use image::ImageError;
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use validator::Validate;
use xredis::RedisPool;

pub mod new;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(project_create)
        .service(project_create_with_id)
        .configure(new::config);
}

#[derive(Error, Debug)]
pub enum CreateError {
    #[error(transparent)]
    Request(crate::routes::ApiError),
    #[error("An unknown database error occurred")]
    SqlxDatabaseError(#[from] sqlx::Error),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] models::DatabaseError),
    #[error("Error while parsing multipart payload: {0}")]
    MultipartError(#[from] actix_multipart::MultipartError),
    #[error("Error while parsing JSON: {0}")]
    SerDeError(#[from] serde_json::Error),
    #[error("Error while validating input: {0}")]
    ValidationError(String),
    #[error("Error while uploading file: {0}")]
    FileHostingError(#[from] FileHostingError),
    #[error("Error while validating uploaded file: {0}")]
    FileValidationError(#[from] crate::validate::ValidationError),
    #[error("{}", .0)]
    MissingValueError(String),
    #[error("Invalid format for image: {0}")]
    InvalidIconFormat(String),
    #[error("Error with multipart data: {0}")]
    InvalidInput(String),
    #[error("Invalid game version: {0}")]
    InvalidGameVersion(String),
    #[error("Invalid loader: {0}")]
    InvalidLoader(String),
    #[error("Invalid category: {0}")]
    InvalidCategory(String),
    #[error("Invalid file type for version file: {0}")]
    InvalidFileType(String),
    #[error("Slug is already taken!")]
    SlugCollision,
    #[error("Authentication Error: {0}")]
    Unauthorized(#[from] AuthenticationError),
    #[error("Authentication Error: {0}")]
    CustomAuthenticationError(String),
    #[error("Image Parsing Error: {0}")]
    ImageError(#[from] ImageError),
    #[error("Project limit reached")]
    LimitReached,
}

impl From<crate::routes::ApiError> for CreateError {
    fn from(value: crate::routes::ApiError) -> Self {
        match value {
            crate::routes::ApiError::Auth(err) => {
                Self::CustomAuthenticationError(format!("{err:#}"))
            }
            crate::routes::ApiError::Request(err) => {
                if err.downcast_ref::<super::projects::validate::ProjectValidationError>().is_some() {
					Self::Request(crate::routes::ApiError::Request(err))
				} else {
					Self::InvalidInput(format!("{err:#}"))
				}
            }
            err => Self::DatabaseError(models::DatabaseError::SchemaError(
                format!("{err:#}"),
            )),
        }
    }
}

impl actix_web::ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::Request(error) => error.status_code(),
            CreateError::SqlxDatabaseError(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CreateError::DatabaseError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::FileHostingError(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CreateError::SerDeError(..) => StatusCode::BAD_REQUEST,
            CreateError::MultipartError(..) => StatusCode::BAD_REQUEST,
            CreateError::MissingValueError(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidIconFormat(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidInput(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidGameVersion(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidLoader(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidCategory(..) => StatusCode::BAD_REQUEST,
            CreateError::InvalidFileType(..) => StatusCode::BAD_REQUEST,
            CreateError::Unauthorized(..) => StatusCode::UNAUTHORIZED,
            CreateError::CustomAuthenticationError(..) => {
                StatusCode::UNAUTHORIZED
            }
            CreateError::SlugCollision => StatusCode::BAD_REQUEST,
            CreateError::ValidationError(..) => StatusCode::BAD_REQUEST,
            CreateError::FileValidationError(..) => StatusCode::BAD_REQUEST,
            CreateError::ImageError(..) => StatusCode::BAD_REQUEST,
            CreateError::LimitReached => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let Self::Request(error) = self {
            return error.error_response();
        }
        HttpResponse::build(self.status_code()).json(ApiError {
            error: match self {
                CreateError::Request(..) => "request_error",
                CreateError::SqlxDatabaseError(..) => "database_error",
                CreateError::DatabaseError(..) => "database_error",
                CreateError::FileHostingError(..) => "file_hosting_error",
                CreateError::SerDeError(..) => "invalid_input",
                CreateError::MultipartError(..) => "invalid_input",
                CreateError::MissingValueError(..) => "invalid_input",
                CreateError::InvalidIconFormat(..) => "invalid_input",
                CreateError::InvalidInput(..) => "invalid_input",
                CreateError::InvalidGameVersion(..) => "invalid_input",
                CreateError::InvalidLoader(..) => "invalid_input",
                CreateError::InvalidCategory(..) => "invalid_input",
                CreateError::InvalidFileType(..) => "invalid_input",
                CreateError::Unauthorized(..) => "unauthorized",
                CreateError::CustomAuthenticationError(..) => "unauthorized",
                CreateError::SlugCollision => "invalid_input",
                CreateError::ValidationError(..) => "invalid_input",
                CreateError::FileValidationError(..) => "invalid_input",
                CreateError::ImageError(..) => "invalid_image",
                CreateError::LimitReached => "limit_reached",
            },
            description: self.to_string(),
            details: None,
        })
    }
}

pub fn default_project_type() -> String {
    "mod".to_string()
}

fn default_requested_status() -> ProjectStatus {
    ProjectStatus::Approved
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct ProjectCreateData {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    #[serde(alias = "mod_name")]
    /// The title or name of the project.
    pub name: String,
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
    pub summary: String,
    #[validate(length(max = 65536))]
    #[serde(alias = "mod_body")]
    /// A long description of the project, in markdown.
    pub description: String,

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

    /// An optional link to the project's license page
    pub license_url: Option<String>,
    /// An optional list of all donation links the project has
    #[serde(default)]
    pub link_urls: HashMap<String, String>,

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
    pub organization_id: Option<OrganizationId>,
}

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct NewGalleryItem {
    /// The name of the multipart item where the gallery media is located
    pub item: String,
    /// Whether the gallery item should show in search or not
    pub featured: bool,
    #[validate(length(min = 1, max = 2048))]
    /// The title of the gallery item
    pub name: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    /// The description of the gallery item
    pub description: Option<String>,
    pub ordering: i64,
}

pub struct UploadedFile {
    pub name: String,
    pub publicity: FileHostPublicity,
}

pub async fn undo_uploads(
    file_host: &dyn FileHost,
    uploaded_files: &[UploadedFile],
) -> Result<(), FileHostingError> {
    for file in uploaded_files {
        file_host.delete_file(&file.name, file.publicity).await?;
    }
    Ok(())
}

/// Create a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	request_body(
		content(("multipart/form-data")),
		description = "Multipart payload containing project metadata and files"
	),
	responses((status = OK, body = Project))
)]
#[post("")]
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
    project_create_internal(
        req,
        payload,
        client,
        redis,
        file_host,
        session_queue,
        kafka_client,
        search_state,
    )
    .await
}

pub async fn project_create_internal(
    req: HttpRequest,
    mut payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    session_queue: Data<AuthQueue>,
    kafka_client: Data<KafkaClientState>,
    search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let project_id: ProjectId =
        models::generate_project_id(&mut transaction).await?.into();

    let result = project_create_inner(
        req,
        &mut payload,
        &mut transaction,
        &**file_host,
        &mut uploaded_files,
        &client,
        &redis,
        &session_queue,
        &kafka_client,
        project_id,
    )
    .await;

    if result.is_err() {
        let undo_result = undo_uploads(&**file_host, &uploaded_files).await;
        let rollback_result = transaction.rollback().await;

        undo_result?;
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
        super::projects::clear_project_cache_and_queue_search(
            &redis,
            &search_state,
            project_id.into(),
            None,
            None,
        )
        .await?;
    }

    result
}

/// Create a project with a specific ID.
///
/// This is a testing endpoint only accessible behind an admin key.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	request_body(
		content(("multipart/form-data")),
		description = "Multipart payload containing project metadata and files"
	),
	responses((status = OK, body = Project))
)]
#[post("/{id}", guard = "admin_key_guard")]
pub async fn project_create_with_id(
    req: HttpRequest,
    mut payload: Multipart,
    client: Data<PgPool>,
    redis: Data<RedisPool>,
    file_host: Data<dyn FileHost>,
    session_queue: Data<AuthQueue>,
    kafka_client: Data<KafkaClientState>,
    search_state: Data<SearchState>,
    path: web::Path<(ProjectId,)>,
) -> Result<HttpResponse, CreateError> {
    let mut transaction = client.begin().await?;
    let mut uploaded_files = Vec::new();

    let (project_id,) = path.into_inner();

    let result = project_create_inner(
        req,
        &mut payload,
        &mut transaction,
        &**file_host,
        &mut uploaded_files,
        &client,
        &redis,
        &session_queue,
        &kafka_client,
        project_id,
    )
    .await;

    if result.is_err() {
        let undo_result = undo_uploads(&**file_host, &uploaded_files).await;
        let rollback_result = transaction.rollback().await;

        undo_result?;
        if let Err(e) = rollback_result {
            return Err(e.into());
        }
    } else {
        transaction.commit().await?;
        super::projects::clear_project_cache_and_queue_search(
            &redis,
            &search_state,
            project_id.into(),
            None,
            None,
        )
        .await?;
    }

    result
}

/*

Project Creation Steps:
- Get logged in user
    Must match the author in the version creation

- Check they have not exceeded their project limit

1. Data
    - Gets "data" field from multipart form; must be first
    - Verification: string lengths
    - Create versions
        - Some shared logic with version creation
        - Create list of VersionBuilders
    - Create ProjectBuilder

2. Upload
    - Icon: check file format & size
        - Upload to S3 & record URL
    - Project files
        - Check for matching version
        - File size limits?
        - Check file type
            - Eventually, malware scan
        - Upload to S3 & create VersionFileBuilder
    -

3. Creation
    - Database stuff
    - Add project data to indexing queue
*/

#[allow(clippy::too_many_arguments)]
async fn project_create_inner(
    req: HttpRequest,
    payload: &mut Multipart,
    transaction: &mut PgTransaction<'_>,
    file_host: &dyn FileHost,
    uploaded_files: &mut Vec<UploadedFile>,
    pool: &PgPool,
    redis: &RedisPool,
    session_queue: &AuthQueue,
    kafka_client: &KafkaClientState,
    project_id: ProjectId,
) -> Result<HttpResponse, CreateError> {
    // The currently logged in user
    let (_, current_user) = get_user_from_headers(
        &req,
        pool,
        redis,
        session_queue,
        Scopes::PROJECT_CREATE,
    )
    .await?;

    require_verified_email(&current_user)?;

    let limits = UserLimits::get_for_projects(&current_user, pool).await?;
    if limits.current >= limits.max {
        return Err(CreateError::LimitReached);
    }

    let all_loaders =
        models::loader_fields::Loader::list(&mut *transaction, redis).await?;

    let project_create_data: ProjectCreateData;
    let mut versions;
    let mut versions_map = std::collections::HashMap::new();
    let mut gallery_urls = Vec::new();
    {
        // The first multipart field must be named "data" and contain a
        // JSON `ProjectCreateData` object.

        let mut field = payload.next().await.map_or_else(
            || {
                Err(CreateError::MissingValueError(String::from(
                    "No `data` field in multipart upload",
                )))
            },
            |m| m.map_err(CreateError::MultipartError),
        )?;

        let name = field.name().ok_or_else(|| {
            CreateError::MissingValueError(String::from("Missing content name"))
        })?;

        if name != "data" {
            return Err(CreateError::InvalidInput(String::from(
                "`data` field must come before file fields",
            )));
        }

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(
                &chunk.map_err(CreateError::MultipartError)?,
            );
        }
        let create_data: ProjectCreateData = serde_json::from_slice(&data)?;

        create_data.validate().map_err(|err| {
            CreateError::InvalidInput(validation_errors_to_string(err, None))
        })?;

		super::projects::validate::require_valid_project(
			crate::validate::project::validate_link_input(
				&create_data.link_urls,
				&create_data.license_id,
				create_data.license_url.as_deref(),
				&create_data.description,
			),
		)?;

        let slug_project_id_option: Option<ProjectId> = serde_json::from_str(
            &format!("\"{}\"", create_data.slug.to_lowercase()),
        )
        .ok();

        if let Some(slug_project_id) = slug_project_id_option {
            let slug_project_id: models::ids::DBProjectId =
                slug_project_id.into();
            let results = sqlx::query!(
                "
                SELECT EXISTS(SELECT 1 FROM mods WHERE id=$1)
                ",
                slug_project_id as models::ids::DBProjectId
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(|e| CreateError::DatabaseError(e.into()))?;

            if results.exists.unwrap_or(false) {
                return Err(CreateError::SlugCollision);
            }
        }

        {
            let results = sqlx::query!(
                "
                SELECT EXISTS(
                    SELECT 1 FROM mods
                    WHERE
                        slug = LOWER($1)
                        OR text_id_lower = LOWER($1)
                )
                ",
                create_data.slug
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(|e| CreateError::DatabaseError(e.into()))?;

            if results.exists.unwrap_or(false) {
                return Err(CreateError::SlugCollision);
            }
        }

        // Create VersionBuilders for the versions specified in `initial_versions`
        versions = Vec::with_capacity(create_data.initial_versions.len());
        for (i, data) in create_data.initial_versions.iter().enumerate() {
            // Create a map of multipart field names to version indices
            for name in &data.file_parts {
                if versions_map.insert(name.to_owned(), i).is_some() {
                    // If the name is already used
                    return Err(CreateError::InvalidInput(String::from(
                        "Duplicate multipart field name",
                    )));
                }
            }
            versions.push(
                create_initial_version(
                    data,
                    project_id,
                    current_user.id,
                    &all_loaders,
                    transaction,
                    redis,
                )
                .await?,
            );
        }

        project_create_data = create_data;
    }

    let mut icon_data = None;

    let mut error = None;
    while let Some(item) = payload.next().await {
        let mut field: Field = item?;

        if error.is_some() {
            continue;
        }

        let result = async {
            let content_disposition = field.content_disposition().unwrap().clone();

            let name = content_disposition.get_name().ok_or_else(|| {
                CreateError::MissingValueError("Missing content name".to_string())
            })?;

            let (file_name, file_extension) =
                super::version_creation::get_name_ext(&content_disposition)?;

            if name == "icon" {
                if icon_data.is_some() {
                    return Err(CreateError::InvalidInput(String::from(
                        "Projects can only have one icon",
                    )));
                }
                // Upload the icon to the cdn
                icon_data = Some(
                    process_icon_upload(
                        uploaded_files,
                        project_id.0,
                        file_extension,
                        file_host,
                        field,
                    )
                    .await?,
                );
                return Ok(());
            }
            if let Some(gallery_items) = &project_create_data.gallery_items {
                if gallery_items.iter().filter(|a| a.featured).count() > 1 {
                    return Err(CreateError::InvalidInput(String::from(
                        "Only one gallery image can be featured.",
                    )));
                }
                if let Some(item) = gallery_items.iter().find(|x| x.item == name) {
                    let data = read_from_field(
                        &mut field,
                        5 * (1 << 20),
                        "Gallery image exceeds the maximum of 5MiB.",
                    )
                    .await?;

                    let (_, file_extension) =
                        super::version_creation::get_name_ext(&content_disposition)?;

                    let url = format!("data/{project_id}/images");
                    let upload_result = upload_image_optimized(
                        &url,
                        FileHostPublicity::Public,
                        data.freeze(),
                        file_extension,
                        Some(350),
                        Some(1.0),
                        file_host,
                    )
                    .await
                    .map_err(|e| CreateError::InvalidIconFormat(e.to_string()))?;

                    uploaded_files.push(UploadedFile {
                        name: upload_result.raw_url_path,
                        publicity: FileHostPublicity::Public,
                    });
                    gallery_urls.push(crate::models::projects::GalleryItem {
                        url: upload_result.url,
                        raw_url: upload_result.raw_url,
                        featured: item.featured,
                        name: item.name.clone(),
                        description: item.description.clone(),
                        created: Utc::now(),
                        ordering: item.ordering,
                    });

                    return Ok(());
                }
            }
            let index = if let Some(i) = versions_map.get(name) {
                *i
            } else {
                return Err(CreateError::InvalidInput(format!(
                    "File `{file_name}` (field {name}) isn't specified in the versions data"
                )));
            };
            // `index` is always valid for these lists
            let created_version = &mut versions[index];
            let version_data = &project_create_data.initial_versions[index];
            // TODO: maybe redundant is this calculation done elsewhere?

            let existing_file_names = created_version
                .files
                .iter()
                .map(|x| x.filename.clone())
                .collect();
            // Upload the new jar file
            super::version_creation::upload_file(
                &mut field,
                file_host,
                version_data.file_parts.len(),
                uploaded_files,
                &mut created_version.files,
                &mut created_version.dependencies,
                &content_disposition,
                project_id,
                created_version.version_id.into(),
                &created_version.version_fields,
                version_data.loaders.clone(),
                version_data.primary_file.is_some(),
                version_data.primary_file.as_deref() == Some(name),
                None,
                existing_file_names,
                transaction,
                redis,
            )
            .await?;

            Ok(())
        }
        .await;

        if result.is_err() {
            error = result.err();
        }
    }

    if let Some(error) = error {
        return Err(error);
    }

    {
        // Check to make sure that all specified files were uploaded
        for (version_data, builder) in project_create_data
            .initial_versions
            .iter()
            .zip(versions.iter())
        {
            if version_data.file_parts.len() != builder.files.len() {
                return Err(CreateError::InvalidInput(String::from(
                    "Some files were specified in initial_versions but not uploaded",
                )));
            }
        }

        // Convert the list of category names to actual categories
        let mut categories =
            Vec::with_capacity(project_create_data.categories.len());
        for category in &project_create_data.categories {
            let ids = models::categories::Category::get_ids(
                category,
                &mut *transaction,
            )
            .await?;
            if ids.is_empty() {
                return Err(CreateError::InvalidCategory(category.clone()));
            }

            // TODO: We should filter out categories that don't match the project type of any of the versions
            // ie: if mod and modpack both share a name this should only have modpack if it only has a modpack as a version
            categories.extend(ids.values());
        }

        let mut additional_categories =
            Vec::with_capacity(project_create_data.additional_categories.len());
        for category in &project_create_data.additional_categories {
            let ids = models::categories::Category::get_ids(
                category,
                &mut *transaction,
            )
            .await?;
            if ids.is_empty() {
                return Err(CreateError::InvalidCategory(category.clone()));
            }
            // TODO: We should filter out categories that don't match the project type of any of the versions
            // ie: if mod and modpack both share a name this should only have modpack if it only has a modpack as a version
            additional_categories.extend(ids.values());
        }

        let mut members = vec![];

        if let Some(organization_id) = project_create_data.organization_id {
            let org = models::DBOrganization::get_id(
                organization_id.into(),
                pool,
                redis,
            )
            .await?
            .ok_or_else(|| {
                CreateError::InvalidInput(
                    "Invalid organization ID specified!".to_string(),
                )
            })?;

            let team_member = models::DBTeamMember::get_from_user_id(
                org.team_id,
                current_user.id.into(),
                pool,
            )
            .await?;

            let perms = OrganizationPermissions::get_permissions_by_role(
                &current_user.role,
                &team_member,
            );

            if !perms.is_some_and(|x| {
                x.contains(OrganizationPermissions::ADD_PROJECT)
            }) {
                return Err(CreateError::CustomAuthenticationError(
                    "You do not have the permissions to create projects in this organization!"
                        .to_string(),
                ));
            }
        } else {
            members.push(models::team_item::TeamMemberBuilder {
                user_id: current_user.id.into(),
                role: crate::models::teams::DEFAULT_ROLE.to_owned(),
                is_owner: true,
                permissions: ProjectPermissions::all(),
                organization_permissions: None,
                accepted: true,
                payouts_split: Decimal::ONE_HUNDRED,
                ordering: 0,
            })
        }
        let team = models::team_item::TeamBuilder { members };

        let team_id = team.insert(&mut *transaction).await?;

        let status;
        if project_create_data.is_draft.unwrap_or(false) {
            status = ProjectStatus::Draft;
        } else {
            status = ProjectStatus::Processing;
            if project_create_data.initial_versions.is_empty() {
                return Err(CreateError::InvalidInput(String::from(
                    "Project submitted for review with no initial versions",
                )));
            }
        }

        let license_id = spdx::Expression::parse(
            &project_create_data.license_id,
        )
        .map_err(|err| {
            CreateError::InvalidInput(format!(
                "Invalid SPDX license identifier: {err}"
            ))
        })?;

        let mut link_urls = vec![];

        let link_platforms =
            models::categories::LinkPlatform::list(&mut *transaction, redis)
                .await?;
        for (platform, url) in &project_create_data.link_urls {
            let platform_id = models::categories::LinkPlatform::get_id(
                platform,
                &mut *transaction,
            )
            .await?
            .ok_or_else(|| {
                CreateError::InvalidInput(format!(
                    "Link platform {} does not exist.",
                    platform.clone()
                ))
            })?;
            let link_platform = link_platforms
                .iter()
                .find(|x| x.id == platform_id)
                .ok_or_else(|| {
                    CreateError::InvalidInput(format!(
                        "Link platform {} does not exist.",
                        platform.clone()
                    ))
                })?;
            link_urls.push(models::project_item::LinkUrl {
                platform_id,
                platform_name: link_platform.name.clone(),
                url: url.clone(),
                donation: link_platform.donation,
            })
        }

        let project_builder_actual = models::project_item::ProjectBuilder {
            project_id: project_id.into(),
            team_id,
            organization_id: project_create_data
                .organization_id
                .map(|x| x.into()),
            name: project_create_data.name,
            summary: project_create_data.summary,
            description: project_create_data.description,
            icon_url: icon_data.clone().map(|x| x.0),
            raw_icon_url: icon_data.clone().map(|x| x.1),

            license_url: project_create_data.license_url,
            categories,
            additional_categories,
            initial_versions: versions,
            status,
            requested_status: Some(project_create_data.requested_status),
            license: license_id.to_string(),
            slug: Some(project_create_data.slug),
            link_urls,
            gallery_items: gallery_urls
                .iter()
                .map(|x| models::project_item::DBGalleryItem {
                    image_url: x.url.clone(),
                    raw_image_url: x.raw_url.clone(),
                    featured: x.featured,
                    name: x.name.clone(),
                    description: x.description.clone(),
                    created: x.created,
                    ordering: x.ordering,
                })
                .collect(),
            color: icon_data.and_then(|x| x.2),
            monetization_status: MonetizationStatus::Monetized,
            components: exp::ProjectSerial::default(),
        };
        let project_builder = project_builder_actual.clone();

        let now = Utc::now();

        let id = project_builder_actual
            .insert(&mut *transaction, redis, file_host, kafka_client)
            .await?;
        DBUser::clear_project_cache(&[current_user.id.into()], redis).await?;

        for image_id in project_create_data.uploaded_images {
            if let Some(db_image) = image_item::DBImage::get(
                image_id.into(),
                &mut *transaction,
                redis,
            )
            .await?
            {
                let image: Image = db_image.into();
                if !matches!(image.context, ImageContext::Project { .. })
                    || image.context.inner_id().is_some()
                {
                    return Err(CreateError::InvalidInput(format!(
                        "Image {image_id} is not unused and in the 'project' context"
                    )));
                }

                sqlx::query!(
                    "
                    UPDATE uploaded_images
                    SET mod_id = $1
                    WHERE id = $2
                    ",
                    id as models::ids::DBProjectId,
                    image_id.0 as i64
                )
                .execute(&mut *transaction)
                .await?;

                image_item::DBImage::clear_cache(image.id.into(), redis)
                    .await?;
            } else {
                return Err(CreateError::InvalidInput(format!(
                    "Image {image_id} does not exist"
                )));
            }
        }

        let thread_id = ThreadBuilder {
            type_: ThreadType::Project,
            members: vec![],
            project_id: Some(id),
            report_id: None,
        }
        .insert(&mut *transaction)
        .await?;

        let loaders = project_builder
            .initial_versions
            .iter()
            .flat_map(|v| v.loaders.clone())
            .unique()
            .collect::<Vec<_>>();
        let (project_types, games) = Loader::list(&mut *transaction, redis)
            .await?
            .into_iter()
            .fold(
                (Vec::new(), Vec::new()),
                |(mut project_types, mut games), loader| {
                    if loaders.contains(&loader.id) {
                        project_types.extend(loader.supported_project_types);
                        games.extend(loader.supported_games);
                    }
                    (project_types, games)
                },
            );

        let response = crate::models::projects::Project {
            id: project_id,
            slug: project_builder.slug.clone(),
            project_types,
            games,
            team_id: team_id.into(),
            organization: project_create_data.organization_id,
            name: project_builder.name.clone(),
            summary: project_builder.summary.clone(),
            description: project_builder.description.clone(),
            published: now,
            updated: now,
            approved: None,
            queued: None,
            status,
            requested_status: project_builder.requested_status,
            moderator_message: None,
            license: License {
                id: project_create_data.license_id.clone(),
                name: "".to_string(),
                url: project_builder.license_url.clone(),
            },
            downloads: 0,
            followers: 0,
            categories: project_create_data.categories,
            additional_categories: project_create_data.additional_categories,
            loaders: vec![],
            versions: project_builder
                .initial_versions
                .iter()
                .map(|v| v.version_id.into())
                .collect::<Vec<_>>(),
            icon_url: project_builder.icon_url.clone(),
            raw_icon_url: project_builder.raw_icon_url.clone(),
            link_urls: project_builder
                .link_urls
                .clone()
                .into_iter()
                .map(|x| (x.platform_name.clone(), Link::from(x)))
                .collect(),
            gallery: gallery_urls,
            color: project_builder.color,
            thread_id: thread_id.into(),
            monetization_status: MonetizationStatus::Monetized,
            // New projects are considered reviewed with respect to side types migrations
            side_types_migration_review_status:
                SideTypesMigrationReviewStatus::Reviewed,
            fields: HashMap::new(), // Fields instantiate to empty
            components: exp::ProjectQuery::default(),
        };

        Ok(HttpResponse::Ok().json(response))
    }
}

async fn create_initial_version(
    version_data: &InitialVersionData,
    project_id: ProjectId,
    author: UserId,
    all_loaders: &[models::loader_fields::Loader],
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<models::version_item::VersionBuilder, CreateError> {
    if version_data.project_id.is_some() {
        return Err(CreateError::InvalidInput(String::from(
            "Found project id in initial version for new project",
        )));
    }

    version_data.validate().map_err(|err| {
        CreateError::ValidationError(validation_errors_to_string(err, None))
    })?;

    // Randomly generate a new id to be used for the version
    let version_id: VersionId =
        models::generate_version_id(transaction).await?.into();

    let loaders = version_data
        .loaders
        .iter()
        .map(|x| {
            all_loaders
                .iter()
                .find(|y| y.loader == x.0)
                .ok_or_else(|| CreateError::InvalidLoader(x.0.clone()))
                .map(|y| y.id)
        })
        .collect::<Result<Vec<models::LoaderId>, CreateError>>()?;

    let loader_fields =
        LoaderField::get_fields(&loaders, &mut *transaction, redis).await?;
    let mut loader_field_enum_values =
        LoaderFieldEnumValue::list_many_loader_fields(
            &loader_fields,
            &mut *transaction,
            redis,
        )
        .await?;

    let version_fields = try_create_version_fields(
        version_id,
        &version_data.fields,
        &loader_fields,
        &mut loader_field_enum_values,
    )?;

    let dependencies = version_data
        .dependencies
        .iter()
        .map(|d| models::version_item::DependencyBuilder {
            version_id: d.version_id.map(|x| x.into()),
            project_id: d.project_id.map(|x| x.into()),
            dependency_type: d.dependency_type.to_string(),
            file_name: None,
        })
        .collect::<Vec<_>>();

    let version = models::version_item::VersionBuilder {
        version_id: version_id.into(),
        project_id: project_id.into(),
        author_id: author.into(),
        name: version_data.version_title.clone(),
        version_number: version_data.version_number.clone(),
        changelog: version_data.version_body.clone().unwrap_or_default(),
        files: Vec::new(),
        dependencies,
        loaders,
        version_fields,
        featured: version_data.featured,
        status: VersionStatus::Listed,
        version_type: version_data.release_channel.to_string(),
        requested_status: None,
        ordering: version_data.ordering,
        components: exp::VersionSerial::default(),
    };

    Ok(version)
}

async fn process_icon_upload(
    uploaded_files: &mut Vec<UploadedFile>,
    id: u64,
    file_extension: &str,
    file_host: &dyn FileHost,
    mut field: Field,
) -> Result<(String, String, Option<u32>), CreateError> {
    let data = read_from_field(
        &mut field,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await?;
    let upload_result = crate::util::img::upload_image_optimized(
        &format!("data/{}", to_base62(id)),
        FileHostPublicity::Public,
        data.freeze(),
        file_extension,
        Some(96),
        Some(1.0),
        file_host,
    )
    .await
    .map_err(|e| CreateError::InvalidIconFormat(e.to_string()))?;

    uploaded_files.push(UploadedFile {
        name: upload_result.raw_url_path,
        publicity: FileHostPublicity::Public,
    });

    uploaded_files.push(UploadedFile {
        name: upload_result.url_path,
        publicity: FileHostPublicity::Public,
    });

    Ok((
        upload_result.url,
        upload_result.raw_url,
        upload_result.color,
    ))
}
