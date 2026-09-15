use crate::database::models as db_models;
use crate::database::models::categories::LinkPlatform;
use crate::database::models::{project_item, version_item};
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::file_hosting::FileHost;
use crate::models::disclosures::ProjectDisclosureType;
use crate::models::projects::{
    Link, MonetizationStatus, Project, ProjectStatus, Version,
};
use crate::models::v2::projects::{
    DonationLink, LegacyProject, LegacySideType, LegacyVersion,
};
use crate::models::v2::search::LegacySearchResults;
use crate::queue::session::AuthQueue;
use crate::routes::v3::projects::ProjectIds;
use crate::routes::{ApiError, v2_reroute, v3};
use crate::search::{SearchBackend, SearchRequest, SearchState};
use crate::util::error::ApiContext as _;
use crate::util::error::Context as _;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
use xredis::RedisPool;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(project_search);
    cfg.service(projects_get);
    cfg.service(projects_edit);
    cfg.service(random_projects_get);
    cfg.service(
        web::scope("/project")
            .service(project_get)
            .service(project_get_check)
            .service(project_delete)
            .service(project_edit)
            .service(project_icon_edit)
            .service(delete_project_icon)
            .service(add_gallery_item)
            .service(edit_gallery_item)
            .service(delete_gallery_item)
            .service(project_follow)
            .service(project_unfollow)
            .service(super::teams::team_members_get_project)
            .service(
                web::scope("/{project_id}")
                    .service(super::versions::version_list)
                    .service(super::versions::version_project_get)
                    .service(dependency_list),
            ),
    );
}

/// Search projects.  
#[utoipa::path(
	tag = "search",
    get,
    operation_id = "searchProjects",
    params(
        ("query" = Option<String>, Query, description = "The query to search for"),
        ("facets" = Option<String>, Query, description = "Search facets JSON"),
        ("index" = Option<String>, Query, description = "Search index to use"),
        ("offset" = Option<String>, Query, description = "Search result offset"),
        ("limit" = Option<String>, Query, description = "Maximum number of search results"),
        ("show_metadata" = Option<bool>, Query, description = "Whether to include search metadata"),
        ("typesense_config" = Option<String>, Query, description = "Typesense request configuration"),
        ("new_filters" = Option<String>, Query, description = "Search filters"),
        ("filters" = Option<String>, Query, description = "Legacy search filters"),
        ("version" = Option<String>, Query, description = "Legacy search version")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = LegacySearchResults),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[get("/search")]
pub async fn project_search(
    web::Query(info): web::Query<SearchRequest>,
    search_backend: web::Data<dyn SearchBackend>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    // Search now uses loader_fields instead of explicit 'client_side' and 'server_side' fields
    // While the backend for this has changed, it doesnt affect much
    // in the API calls except that 'versions:x' is now 'game_versions:x'
    let facets: Option<Vec<Vec<String>>> = if let Some(facets) = info.facets {
        let facets = serde_json::from_str::<Vec<Vec<String>>>(&facets)
            .wrap_request_err("deserializing JSON data")?;

        Some(
            facets
                .into_iter()
                .map(|facet| {
                    facet
                        .into_iter()
                        .map(|facet| {
                            if let Some((key, operator, val)) =
                                parse_facet(&facet)
                            {
                                format!(
                                    "{}{}{}",
                                    match key.as_str() {
                                        "versions" => "game_versions",
                                        "project_type" => "project_types",
                                        "title" => "name",
                                        x => x,
                                    },
                                    operator,
                                    val
                                )
                            } else {
                                facet
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect(),
        )
    } else {
        None
    };

    let info = SearchRequest {
        facets: facets.and_then(|x| serde_json::to_string(&x).ok()),
        ..info
    };

    let results = search_backend
        .search_for_project(&info, &redis)
        .await
        .wrap_api_err("searching projects")?;

    let results = LegacySearchResults::from(results);

    Ok(HttpResponse::Ok().json(results))
}

/// Parses a facet into a key, operator, and value
fn parse_facet(facet: &str) -> Option<(String, String, String)> {
    let mut key = String::new();
    let mut operator = String::new();
    let mut val = String::new();

    let mut iterator = facet.chars();
    while let Some(char) = iterator.next() {
        match char {
            ':' | '=' => {
                operator.push(char);
                val = iterator.collect::<String>();
                return Some((key, operator, val));
            }
            '<' | '>' => {
                operator.push(char);
                if let Some(next_char) = iterator.next() {
                    if next_char == '=' {
                        operator.push(next_char);
                    } else {
                        val.push(next_char);
                    }
                }
                val.push_str(&iterator.collect::<String>());
                return Some((key, operator, val));
            }
            ' ' => continue,
            _ => key.push(char),
        }
    }

    None
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct RandomProjects {
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
}

/// Get random projects.  
#[utoipa::path(
	tag = "projects",
    get,
    operation_id = "randomProjects",
    params(
        ("count" = u32, Query, description = "Number of projects to return")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = Vec<LegacyProject>),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[get("/projects_random")]
pub async fn random_projects_get(
    web::Query(count): web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let count = v3::projects::RandomProjects {
        count: count.count,
        project_type: None,
    };

    let response = v3::projects::random_projects_get(
        web::Query(count),
        pool.clone(),
        redis.clone(),
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
    .or_else(v2_reroute::flatten_404_error)
    .wrap_api_err("flattening v2 not-found response")?;
    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects =
                LegacyProject::from_many(project, &pool, &redis)
                    .await
                    .wrap_internal_err(
                        "executing `LegacyProject::from_many`",
                    )?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

/// Get multiple projects by ID or slug.  
#[utoipa::path(
	tag = "projects",
    get,
    operation_id = "getProjects",
    params(
        ("ids" = String, Query, description = "The JSON array of project IDs or slugs")
    ),
    responses((status = 200, description = "Expected response to a valid request", body = Vec<LegacyProject>))
)]
#[get("/projects")]
pub async fn projects_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Call V3 project creation
    let response = v3::projects::projects_get(
        req,
        web::Query(ids),
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
    .or_else(v2_reroute::flatten_404_error)
    .wrap_api_err("flattening v2 not-found response")?;

    // Convert response to V2 format
    match v2_reroute::extract_ok_json::<Vec<Project>>(response).await {
        Ok(project) => {
            let legacy_projects =
                LegacyProject::from_many(project, &pool, &redis)
                    .await
                    .wrap_internal_err(
                        "executing `LegacyProject::from_many`",
                    )?;
            Ok(HttpResponse::Ok().json(legacy_projects))
        }
        Err(response) => Ok(response),
    }
}

/// Get a project by ID or slug.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    get,
    operation_id = "getProject",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = LegacyProject),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{id}")]
pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Convert V2 data to V3 data
    // Call V3 project creation
    let project = match v3::projects::project_get_internal(
        req,
        info,
        pool.clone(),
        redis.clone(),
        session_queue,
    )
    .await
    {
        Ok(resp) => resp.0,
        Err(ApiError::NotFound(_)) => {
            return Ok(HttpResponse::NotFound().body(""));
        }
        Err(err) => return Err(err),
    };

    // Convert response to V2 format
    let version_item = match project.versions.first() {
        Some(vid) => {
            version_item::DBVersion::get((*vid).into(), &**pool, &redis)
                .await
                .wrap_internal_err("fetching version from database")?
        }
        None => None,
    };
    let has_archived_disclosure =
        db_models::DBProjectDisclosure::projects_with_type(
            ProjectDisclosureType::Archived,
            &[project.id.into()],
            &**pool,
        )
        .await
        .wrap_internal_err("fetching archival disclosure status")?
        .contains(&project.id.into());
    let mut project = LegacyProject::from(project, version_item);
    if has_archived_disclosure && project.status == ProjectStatus::Approved {
        project.status = ProjectStatus::Archived;
    }
    Ok(HttpResponse::Ok().json(project))
}

//checks the validity of a project id or slug
/// Check that a project ID or slug exists.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    get,
    operation_id = "checkProjectValidity",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = v3::projects::ProjectCheckResponse),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/{id}/check")]
pub async fn project_get_check(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    // Returns an id only, do not need to convert
    v3::projects::project_get_check_internal(info, pool, redis)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, utoipa::ToSchema)]
struct DependencyInfo {
    pub projects: Vec<LegacyProject>,
    pub versions: Vec<LegacyVersion>,
}

/// Get dependency projects and versions for a project.  
#[utoipa::path(
	context_path = "/project/{project_id}",
	tag = "projects",
    get,
    operation_id = "getDependencies",
    params(
        ("project_id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = DependencyInfo),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    )
)]
#[get("/dependencies")]
pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // TODO: tests, probably
    let response = v3::projects::dependency_list_internal(
        req,
        info,
        pool.clone(),
        ro_pool,
        redis.clone(),
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
    .wrap_api_err("flattening v2 not-found response")?;

    match v2_reroute::extract_ok_json::<
        crate::routes::v3::projects::DependencyInfo,
    >(response)
    .await
    {
        Ok(dependency_info) => {
            let converted_projects = LegacyProject::from_many(
                dependency_info.projects,
                &pool,
                &redis,
            )
            .await
            .wrap_internal_err(
                "converting dependency projects to legacy responses",
            )?;
            let converted_versions = dependency_info
                .versions
                .into_iter()
                .map(LegacyVersion::from)
                .collect();

            Ok(HttpResponse::Ok().json(DependencyInfo {
                projects: converted_projects,
                versions: converted_versions,
            }))
        }
        Err(response) => Ok(response),
    }
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EditProject {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub title: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub description: Option<String>,
    #[validate(length(max = 65536))]
    pub body: Option<String>,
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub license_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
    #[validate(nested)]
    pub donation_urls: Option<Vec<DonationLink>>,
    pub license_id: Option<String>,
    pub client_side: Option<LegacySideType>,
    pub server_side: Option<LegacySideType>,
    #[validate(
        length(min = 3, max = 64),
        regex(path = *crate::util::validate::RE_URL_SAFE)
    )]
    pub slug: Option<String>,
    pub status: Option<ProjectStatus>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub requested_status: Option<Option<ProjectStatus>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 2000))]
    pub moderation_message: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(max = 65536))]
    pub moderation_message_body: Option<Option<String>>,
    pub monetization_status: Option<MonetizationStatus>,
}

/// Update a project.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    patch,
    operation_id = "modifyProject",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    request_body = EditProject,
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/{id}")]
#[allow(clippy::too_many_arguments)]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    new_project: web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let v2_new_project = new_project.into_inner();
    let client_side = v2_new_project.client_side;
    let server_side = v2_new_project.server_side;
    let new_slug = v2_new_project.slug.clone();

    // TODO: Some kind of handling here to ensure project type is fine.
    // We expect the version uploaded to be of loader type modpack, but there might  not be a way to check here for that.
    // After all, theoretically, they could be creating a genuine 'fabric' mod, and modpack no longer carries information on whether its a mod or modpack,
    // as those are out to the versions.

    // Ideally this would, if the project 'should' be a modpack:
    // - change the loaders to mrpack only
    // - add categories to the project for the corresponding loaders

    let mut new_links = HashMap::new();
    if let Some(issues_url) = v2_new_project.issues_url {
        if let Some(issues_url) = issues_url {
            new_links.insert("issues".to_string(), Some(issues_url));
        } else {
            new_links.insert("issues".to_string(), None);
        }
    }

    if let Some(source_url) = v2_new_project.source_url {
        if let Some(source_url) = source_url {
            new_links.insert("source".to_string(), Some(source_url));
        } else {
            new_links.insert("source".to_string(), None);
        }
    }

    if let Some(wiki_url) = v2_new_project.wiki_url {
        if let Some(wiki_url) = wiki_url {
            new_links.insert("wiki".to_string(), Some(wiki_url));
        } else {
            new_links.insert("wiki".to_string(), None);
        }
    }

    if let Some(discord_url) = v2_new_project.discord_url {
        if let Some(discord_url) = discord_url {
            new_links.insert("discord".to_string(), Some(discord_url));
        } else {
            new_links.insert("discord".to_string(), None);
        }
    }

    // In v2, setting donation links resets all other donation links
    // (resetting to the new ones)
    if let Some(donation_urls) = v2_new_project.donation_urls {
        crate::models::v2::projects::validate_donation_platforms(
            &donation_urls,
        )?;
        // Fetch current donation links from project so we know what to delete
        let fetched_example_project =
            project_item::DBProject::get(&info.0, &**pool, &redis)
                .await
                .wrap_api_err("fetching project from database")?;
        let donation_links = fetched_example_project
            .map(|x| {
                x.urls
                    .into_iter()
                    .filter_map(|l| {
                        if l.donation {
                            Some(Link::from(l)) // TODO: tests
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Set existing donation links to None
        for old_link in donation_links {
            new_links.insert(old_link.platform, None);
        }

        // Add new donation links
        for donation_url in donation_urls {
            new_links.insert(donation_url.id, Some(donation_url.url));
        }
    }

    let new_project = v3::projects::EditProject {
        name: v2_new_project.title,
        summary: v2_new_project.description, // Description becomes summary
        description: v2_new_project.body,    // Body becomes description
        categories: v2_new_project.categories,
        additional_categories: v2_new_project.additional_categories,
        license_url: v2_new_project.license_url,
        link_urls: (!new_links.is_empty()).then_some(new_links),
        license_id: v2_new_project.license_id,
        slug: v2_new_project.slug,
        status: v2_new_project.status,
        requested_status: v2_new_project.requested_status,
        moderation_message: v2_new_project.moderation_message,
        moderation_message_body: v2_new_project.moderation_message_body,
        monetization_status: v2_new_project.monetization_status,
        side_types_migration_review_status: None, // Not to be exposed in v2
        // None of the below is present in v2
        loader_fields: HashMap::new(),
        minecraft_server: None,
        minecraft_java_server: None,
        minecraft_bedrock_server: None,
    };

    // This returns 204 or failure so we don't need to do anything with it
    let project_id = info.clone().0;
    let mut response = v3::projects::project_edit_internal(
        req.clone(),
        info,
        pool.clone(),
        web::Json(new_project),
        redis.clone(),
        session_queue.clone(),
        search_state.clone(),
        true,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)?;

    // If client and server side were set, we will call
    // the version setting route for each version to set the side types for each of them.
    if response.status().is_success()
        && (client_side.is_some() || server_side.is_some())
    {
        let project_item = project_item::DBProject::get(
            &new_slug.unwrap_or(project_id),
            &**pool,
            &redis,
        )
        .await
        .wrap_api_err("fetching project from database")?;
        let version_ids = project_item.map(|x| x.versions).unwrap_or_default();
        let versions =
            version_item::DBVersion::get_many(&version_ids, &**pool, &redis)
                .await
                .wrap_internal_err("fetching versions from database")?;
        for version in versions {
            let version = Version::from(version);
            let mut fields = version.fields;
            let (current_client_side, current_server_side) =
                v2_reroute::convert_v3_side_types_to_v2_side_types(
                    &fields, None,
                );
            let client_side = client_side.unwrap_or(current_client_side);
            let server_side = server_side.unwrap_or(current_server_side);
            fields.extend(v2_reroute::convert_v2_side_types_to_v3_side_types(
                client_side,
                server_side,
            ));

            response = v3::versions::version_edit_helper(
                req.clone(),
                (version.id,),
                pool.clone(),
                redis.clone(),
                v3::versions::EditVersion {
                    fields,
                    ..Default::default()
                },
                session_queue.clone(),
                search_state.clone(),
            )
            .await
            .wrap_api_err("editing project")?;
        }
    }
    Ok(response)
}

#[derive(Deserialize, Validate, utoipa::ToSchema)]
pub struct BulkEditProject {
    #[validate(length(max = 3))]
    pub categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_categories: Option<Vec<String>>,
    pub remove_categories: Option<Vec<String>>,

    #[validate(length(max = 256))]
    pub additional_categories: Option<Vec<String>>,
    #[validate(length(max = 3))]
    pub add_additional_categories: Option<Vec<String>>,
    pub remove_additional_categories: Option<Vec<String>>,

    #[validate(nested)]
    pub donation_urls: Option<Vec<DonationLink>>,
    #[validate(nested)]
    pub add_donation_urls: Option<Vec<DonationLink>>,
    #[validate(nested)]
    pub remove_donation_urls: Option<Vec<DonationLink>>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub issues_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub source_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub wiki_url: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(
        custom(function = "crate::util::validate::validate_url"),
        length(max = 2048)
    )]
    pub discord_url: Option<Option<String>>,
}

/// Bulk-edit multiple projects.  
#[utoipa::path(
	tag = "projects",
    patch,
    operation_id = "patchProjects",
    params(
        ("ids" = String, Query, description = "The JSON array of project IDs or slugs")
    ),
    request_body = BulkEditProject,
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/projects")]
pub async fn projects_edit(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    bulk_edit_project: web::Json<BulkEditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let bulk_edit_project = bulk_edit_project.into_inner();

    let mut link_urls = HashMap::new();

    // If we are *setting* donation links, we will set every possible donation link to None, as
    // setting will delete all of them then 're-add' the ones we want to keep
    if let Some(donation_url) = bulk_edit_project.donation_urls {
        crate::models::v2::projects::validate_donation_platforms(
            &donation_url,
        )?;
        let link_platforms = LinkPlatform::list(&**pool, &redis)
            .await
            .wrap_internal_err("fetching link platform from Redis")?;
        for link in link_platforms {
            if link.donation {
                link_urls.insert(link.name, None);
            }
        }
        // add
        for donation_url in donation_url {
            link_urls.insert(donation_url.id, Some(donation_url.url));
        }
    }

    // For every delete, we will set the link to None
    if let Some(donation_url) = bulk_edit_project.remove_donation_urls {
        crate::models::v2::projects::validate_donation_platforms(
            &donation_url,
        )?;
        for donation_url in donation_url {
            link_urls.insert(donation_url.id, None);
        }
    }

    // For every add, we will set the link to the new url
    if let Some(donation_url) = bulk_edit_project.add_donation_urls {
        crate::models::v2::projects::validate_donation_platforms(
            &donation_url,
        )?;
        for donation_url in donation_url {
            link_urls.insert(donation_url.id, Some(donation_url.url));
        }
    }

    if let Some(issue_url) = bulk_edit_project.issues_url {
        if let Some(issue_url) = issue_url {
            link_urls.insert("issues".to_string(), Some(issue_url));
        } else {
            link_urls.insert("issues".to_string(), None);
        }
    }

    if let Some(source_url) = bulk_edit_project.source_url {
        if let Some(source_url) = source_url {
            link_urls.insert("source".to_string(), Some(source_url));
        } else {
            link_urls.insert("source".to_string(), None);
        }
    }

    if let Some(wiki_url) = bulk_edit_project.wiki_url {
        if let Some(wiki_url) = wiki_url {
            link_urls.insert("wiki".to_string(), Some(wiki_url));
        } else {
            link_urls.insert("wiki".to_string(), None);
        }
    }

    if let Some(discord_url) = bulk_edit_project.discord_url {
        if let Some(discord_url) = discord_url {
            link_urls.insert("discord".to_string(), Some(discord_url));
        } else {
            link_urls.insert("discord".to_string(), None);
        }
    }

    // This returns NoContent or failure so we don't need to do anything with it
    v3::projects::projects_edit(
        req,
        web::Query(ids),
        pool.clone(),
        web::Json(v3::projects::BulkEditProject {
            categories: bulk_edit_project.categories,
            add_categories: bulk_edit_project.add_categories,
            remove_categories: bulk_edit_project.remove_categories,
            additional_categories: bulk_edit_project.additional_categories,
            add_additional_categories: bulk_edit_project
                .add_additional_categories,
            remove_additional_categories: bulk_edit_project
                .remove_additional_categories,
            link_urls: Some(link_urls),
        }),
        redis,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct Extension {
    pub ext: String,
}

/// Change a project's icon.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    patch,
    operation_id = "changeProjectIcon",
    params(
        ("id" = String, Path, description = "The ID or slug of the project"),
        ("ext" = String, Query, description = "Image extension (png, jpg, jpeg, bmp, gif, webp, svg, svgz, rgb)")
    ),
    request_body(
        content(
            ("image/png"),
            ("image/jpeg"),
            ("image/bmp"),
            ("image/gif"),
            ("image/webp"),
            ("image/svg+xml")
        )
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error")
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/{id}/icon")]
#[allow(clippy::too_many_arguments)]
pub async fn project_icon_edit(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::project_icon_edit_internal(
        web::Query(v3::projects::Extension { ext: ext.ext }),
        req,
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Delete a project's icon.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    delete,
    operation_id = "deleteProjectIcon",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[delete("/{id}/icon")]
pub async fn delete_project_icon(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::delete_project_icon_internal(
        req,
        info,
        pool,
        redis,
        file_host,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct GalleryCreateQuery {
    pub featured: bool,
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

/// Add a gallery image to a project.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    post,
    operation_id = "addGalleryImage",
    params(
        ("id" = String, Path, description = "The ID or slug of the project"),
        ("ext" = String, Query, description = "Image extension (png, jpg, jpeg, bmp, gif, webp, svg, svgz, rgb)"),
        ("featured" = bool, Query, description = "Whether this image is featured"),
        ("title" = Option<String>, Query, description = "Image title"),
        ("description" = Option<String>, Query, description = "Image description"),
        ("ordering" = Option<i64>, Query, description = "Image ordering")
    ),
    request_body(
        content(
            ("image/png"),
            ("image/jpeg"),
            ("image/bmp"),
            ("image/gif"),
            ("image/webp"),
            ("image/svg+xml")
        )
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[post("/{id}/gallery")]
#[allow(clippy::too_many_arguments)]
pub async fn add_gallery_item(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    web::Query(item): web::Query<GalleryCreateQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::add_gallery_item_internal(
        web::Query(v3::projects::Extension { ext: ext.ext }),
        req,
        web::Query(v3::projects::GalleryCreateQuery {
            featured: item.featured,
            name: item.title,
            description: item.description,
            ordering: item.ordering,
        }),
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct GalleryEditQuery {
    /// The url of the gallery item to edit
    pub url: String,
    pub featured: Option<bool>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 255))]
    pub title: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<Option<String>>,
    pub ordering: Option<i64>,
}

/// Update a gallery image.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    patch,
    operation_id = "modifyGalleryImage",
    params(
        ("id" = String, Path, description = "The ID or slug of the project"),
        ("url" = String, Query, description = "URL of the image to edit"),
        ("featured" = Option<bool>, Query, description = "Whether this image is featured"),
        ("title" = Option<Option<String>>, Query, description = "Image title"),
        ("description" = Option<Option<String>>, Query, description = "Image description"),
        ("ordering" = Option<i64>, Query, description = "Image ordering")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        ),
        (
            status = 404,
            description = "The requested item(s) were not found or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[patch("/{id}/gallery")]
pub async fn edit_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::edit_gallery_item_internal(
        req,
        web::Query(v3::projects::GalleryEditQuery {
            url: item.url,
            featured: item.featured,
            name: item.title,
            description: item.description,
            ordering: item.ordering,
        }),
        pool,
        redis,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct GalleryDeleteQuery {
    pub url: String,
}

/// Delete a gallery image.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    delete,
    operation_id = "deleteGalleryImage",
    params(
        ("id" = String, Path, description = "The ID or slug of the project"),
        ("url" = String, Query, description = "URL of the image to delete")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_WRITE"]))
)]
#[delete("/{id}/gallery")]
pub async fn delete_gallery_item(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryDeleteQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::delete_gallery_item_internal(
        req,
        web::Query(v3::projects::GalleryDeleteQuery { url: item.url }),
        pool,
        redis,
        file_host,
        session_queue,
        search_state,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}

/// Delete a project by ID or slug.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    delete,
    operation_id = "deleteProject",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["PROJECT_DELETE"]))
)]
#[delete("/{id}")]
pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::project_delete_internal(
        req,
        info,
        pool,
        redis,
        session_queue,
        search_state,
    )
    .await
    .map(|()| HttpResponse::NoContent().body(""))
    .or_else(v2_reroute::flatten_404_error)
}

/// Follow a project.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    post,
    operation_id = "followProject",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_WRITE"]))
)]
#[post("/{id}/follow")]
pub async fn project_follow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::project_follow_internal(req, info, pool, redis, session_queue)
        .await
        .or_else(v2_reroute::flatten_404_error)
}

/// Unfollow a project.  
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
    delete,
    operation_id = "unfollowProject",
    params(
        ("id" = String, Path, description = "The ID or slug of the project")
    ),
    responses(
        (status = NO_CONTENT, description = "Expected response to a valid request"),
        (status = 400, description = "Request was invalid, see given error"),
        (
            status = 401,
            description = "Incorrect token scopes or no authorization to access the requested item(s)"
        )
    ),
    security(("bearer_auth" = ["USER_WRITE"]))
)]
#[delete("/{id}/follow")]
pub async fn project_unfollow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    // Returns NoContent, so no need to convert
    v3::projects::project_unfollow_internal(
        req,
        info,
        pool,
        redis,
        session_queue,
    )
    .await
    .or_else(v2_reroute::flatten_404_error)
}
