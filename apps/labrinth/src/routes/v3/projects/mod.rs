use crate::util::error::ApiContext as _;
use std::any::type_name;
use std::cmp::Reverse;
use std::collections::HashMap;

use crate::auth::checks::{filter_visible_versions, is_visible_project};
use crate::auth::{filter_visible_projects, get_user_from_headers};
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::project_item::{
    DBGalleryItem, DBModCategory, ProjectQueryResult,
};
use crate::database::models::thread_item::ThreadMessageBuilder;
use crate::database::models::{
    DBModerationLock, DBProjectId, DBTeamMember, ids as db_ids, image_item,
};
use crate::database::{self, models as db_models};
use crate::database::{PgPool, PgTransaction, ReadOnlyPgPool};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::disclosures::{
    DisclosureLockStatus, ProjectDisclosure, ProjectDisclosureType,
};
use crate::models::ids::{ProjectId, VersionId};
use crate::models::images::ImageContext;
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
    MonetizationStatus, Project, ProjectStatus, SideTypesMigrationReviewStatus,
};
use crate::models::teams::{DEFAULT_ROLE, ProjectPermissions};
use crate::models::threads::MessageBody;
use crate::models::users::{DELETED_USER, User};
use crate::models::{self, exp};
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::routes::internal::delphi;
use crate::search::{
    SearchBackend, SearchQuery, SearchRequest, SearchResults, SearchState,
};
use crate::util::error::Context;
use crate::util::img;
use crate::util::img::{delete_old_images, upload_image_optimized};
use crate::util::routes::read_limited_from_payload;
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use chrono::Utc;
use eyre::eyre;
use futures::TryStreamExt;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;
use xredis::RedisPool;

pub mod validate;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(project_search)
        .service(project_search_post)
        .service(projects_get_route)
        .service(projects_edit_route)
        .service(random_projects_get_route);
}

pub fn project_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(project_get)
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
        .service(project_get_organization)
        .service(super::teams::team_members_get_project)
        .service(super::versions::version_list)
        .service(super::versions::version_project_get)
        .service(dependency_list)
        .service(validate::validate);
}

pub async fn clear_project_cache_and_queue_search(
    redis: &RedisPool,
    search_state: &SearchState,
    project_id: db_ids::DBProjectId,
    slug: Option<String>,
    clear_dependencies: Option<bool>,
) -> Result<(), ApiError> {
    db_models::DBProject::clear_cache(
        project_id,
        slug,
        clear_dependencies,
        redis,
    )
    .await
    .wrap_internal_err("clearing cached data from Redis")?;

    search_state
        .queue
        .push_project_change(project_id.into())
        .await;

    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct RandomProjects {
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
    pub project_type: Option<String>,
}

#[utoipa::path(
	tag = "projects",
	params(
		("count" = u32, Query),
		("project_type" = Option<String>, Query),
	),
	responses((status = OK))
)]
#[get("/projects_random")]
pub async fn random_projects_get_route(
    count: web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    random_projects_get(count, pool, redis).await
}

// Filtered candidates are sparser and unevenly spaced, so the nearest-point pick
// tends to repeat; oversample a neighborhood and shuffle it down to counter that.
const RANDOM_PROJECT_TYPE_OVERSAMPLE_FACTOR: u32 = 20;

pub async fn random_projects_get(
    web::Query(params): web::Query<RandomProjects>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    params
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let statuses = crate::models::projects::ProjectStatus::iterator()
        .filter(|x| x.is_searchable())
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let mut project_ids = if let Some(project_type) = &params.project_type {
        let fetch_limit = params.count * RANDOM_PROJECT_TYPE_OVERSAMPLE_FACTOR;

        sqlx::query!(
            // IDs are randomly generated (see the `generate_ids` macro), so fetching a
            // number of mods nearest to a random point in the ID space is equivalent to
            // random sampling
            "WITH random_id_point AS (
                SELECT POINT(RANDOM() * ((SELECT MAX(id) FROM mods) - (SELECT MIN(id) FROM mods) + 1) + (SELECT MIN(id) FROM mods), 0) AS point
            )
            SELECT id FROM mods
            WHERE status = ANY($1)
            AND EXISTS (
                SELECT 1 FROM versions v
                INNER JOIN loaders_versions lv ON v.id = lv.version_id
                INNER JOIN loaders_project_types lpt ON lpt.joining_loader_id = lv.loader_id
                INNER JOIN project_types pt ON pt.id = lpt.joining_project_type_id
                WHERE v.mod_id = mods.id AND pt.name = $3
                -- prevents decorrelation, so this stops at the first match instead
                -- of scanning all versions before the outer sort/limit applies
                OFFSET 0
            )
            ORDER BY POINT(id, 0) <-> (SELECT point FROM random_id_point)
            LIMIT $2",
            &statuses,
            fetch_limit as i32,
            project_type,
        )
        .fetch(&**pool)
        .map_ok(|m| db_ids::DBProjectId(m.id))
        .try_collect::<Vec<_>>()
        .await
        .wrap_internal_err("querying random project IDs")?
    } else {
        sqlx::query!(
            // IDs are randomly generated (see the `generate_ids` macro), so fetching a
            // number of mods nearest to a random point in the ID space is equivalent to
            // random sampling
            "WITH random_id_point AS (
                SELECT POINT(RANDOM() * ((SELECT MAX(id) FROM mods) - (SELECT MIN(id) FROM mods) + 1) + (SELECT MIN(id) FROM mods), 0) AS point
            )
            SELECT id FROM mods
            WHERE status = ANY($1)
            ORDER BY POINT(id, 0) <-> (SELECT point FROM random_id_point)
            LIMIT $2",
            &statuses,
            params.count as i32,
        )
        .fetch(&**pool)
        .map_ok(|m| db_ids::DBProjectId(m.id))
        .try_collect::<Vec<_>>()
        .await
        .wrap_internal_err("querying random project IDs")?
    };

    if params.project_type.is_some() {
        project_ids.shuffle(&mut rand::thread_rng());
        project_ids.truncate(params.count as usize);
    }

    let projects_data =
        db_models::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await
            .wrap_api_err("fetching projects by ID")?
            .into_iter()
            .map(Project::from)
            .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(projects_data))
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectIds {
    pub ids: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectCheckResponse {
    pub id: ProjectId,
}

#[utoipa::path(
	tag = "projects",
	params(("ids" = String, Query)),
	responses((status = OK))
)]
#[get("/projects")]
pub async fn projects_get_route(
    req: HttpRequest,
    ids: web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    projects_get(req, ids, pool, redis, session_queue).await
}

pub async fn projects_get(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let ids = serde_json::from_str::<Vec<&str>>(&ids.ids)
        .wrap_request_err("deserializing JSON data")?;
    let projects_data = db_models::DBProject::get_many(&ids, &**pool, &redis)
        .await
        .wrap_api_err("fetching requested projects")?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    let projects =
        filter_visible_projects(projects_data, &user_option, &pool, false)
            .await
            .wrap_api_err("filtering visible projects")?;

    Ok(HttpResponse::Ok().json(projects))
}

/// Get a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = OK, body = Project))
)]
#[get("/{id}")]
pub async fn project_get(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Project>, ApiError> {
    project_get_internal(req, info, pool, redis, session_queue).await
}

pub async fn project_get_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<Project>, ApiError> {
    let (string,) = info.into_inner();

    let project_data = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_internal_err("failed to fetch project")?;
    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|(_, user)| user)
    .ok();

    if let Some(data) = project_data
        && is_visible_project(&data.inner, &user_option, &pool, false)
            .await
            .wrap_internal_err("failed to check project visibility")?
    {
        return Ok(web::Json(Project::from(data)));
    }
    Err(ApiError::NotFound(eyre::eyre!("resource not found")))
}

#[derive(Debug, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct EditProject {
    #[validate(
        length(min = 3, max = 64),
        custom(function = "crate::util::validate::validate_name")
    )]
    pub name: Option<String>,
    #[validate(length(min = 3, max = 256))]
    pub summary: Option<String>,
    #[validate(length(max = 65536))]
    pub description: Option<String>,
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
    pub license_url: Option<Option<String>>,
    // <name, url> (leave url empty to delete)
    #[validate(custom(
        function = "crate::util::validate::validate_url_hashmap_optional_values"
    ))]
    pub link_urls: Option<HashMap<String, Option<String>>>,
    pub license_id: Option<String>,
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
    pub side_types_migration_review_status:
        Option<SideTypesMigrationReviewStatus>,
    #[serde(flatten)]
    pub loader_fields: HashMap<String, serde_json::Value>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub minecraft_server: Option<Option<exp::minecraft::ServerProjectEdit>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub minecraft_java_server:
        Option<Option<exp::minecraft::JavaServerProjectEdit>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub minecraft_bedrock_server:
        Option<Option<exp::minecraft::BedrockServerProjectEdit>>,
}

#[allow(clippy::too_many_arguments)]
/// Update a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = NO_CONTENT))
)]
#[patch("/{id}")]
pub async fn project_edit(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    web::Json(new_project): web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    project_edit_internal(
        req,
        info,
        pool,
        web::Json(new_project),
        redis,
        session_queue,
        search_state,
        false,
    )
    .await
}

pub async fn project_edit_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    web::Json(new_project): web::Json<EditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
    sync_archival_disclosure: bool,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    new_project.validate().map_err(|err| {
        let message =
            crate::util::validate::validation_errors_to_string(err, None)
                .replacen("Field ", "field ", 1);
        ApiError::Request(eyre!(message))
    })?;

    let Some(mut project_item) =
        db_models::DBProject::get(&info.into_inner().0, &**pool, &redis)
            .await
            .wrap_api_err("fetching project")?
    else {
        return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
    };

    let id = project_item.inner.id;

    let (team_member, organization_team_member) =
        db_models::DBTeamMember::get_for_project_permissions(
            &project_item.inner,
            user.id.into(),
            &**pool,
        )
        .await
        .wrap_internal_err("fetching team member from database")?;

    let Some(perms) = ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    ) else {
        return Err(ApiError::Auth(eyre::eyre!(
            "You do not have permission to edit this project!",
        )));
    };

    let submit_for_review = new_project.status
        == Some(ProjectStatus::Processing)
        && !user.role.is_mod();
    let leave_review = matches!(
        new_project.status,
        Some(ProjectStatus::Draft | ProjectStatus::Rejected)
    );
    let validate_for_review = submit_for_review
        || (project_item.inner.status == ProjectStatus::Processing
            && !leave_review);
    if submit_for_review {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre!(
                "you do not have permission to submit this project for review"
            )));
        }
        if project_item.inner.status.is_approved() {
            return Err(ApiError::Auth(eyre!(
                "you do not have permission to submit this project for review"
            )));
        }
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    if let Some(name) = &new_project.name {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the name of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET name = $1
            WHERE (id = $2)
            ",
            name.trim(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(summary) = &new_project.summary {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the summary of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET summary = $1
            WHERE (id = $2)
            ",
            summary,
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(status) = &new_project.status
        && !submit_for_review
    {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the status of this project!",
            )));
        }

        let archival_disclosure =
            db_models::DBProjectDisclosure::get_many_for_project(
                project_item.inner.id,
                false,
                &mut transaction,
            )
            .await
            .wrap_internal_err("failed to fetch project disclosures")?
            .into_iter()
            .find(|disclosure| {
                matches!(
                    disclosure.disclosure,
                    ProjectDisclosure::Archived { .. }
                )
            });
        let has_archived_disclosure = archival_disclosure.is_some();

        if status == &ProjectStatus::Archived {
            if !has_archived_disclosure {
                db_models::DBProjectDisclosure {
                    project_id: project_item.inner.id,
                    disclosure: ProjectDisclosure::Archived { note: None },
                    updated_at: Utc::now(),
                    updated_by: user.id.into(),
                    set_by_moderator: user.role.is_mod(),
                    deleted_at: None,
                    lock_status: DisclosureLockStatus::Unlocked,
                }
                .upsert(&mut transaction)
                .await
                .wrap_internal_err("failed to upsert archival disclosure")?;
            }
        } else {
            if !(user.role.is_mod()
                || !project_item.inner.status.is_approved()
                    && status == &ProjectStatus::Processing
                || project_item.inner.status.is_approved()
                    && status.can_be_requested())
            {
                return Err(ApiError::Auth(eyre::eyre!(
                    "You don't have permission to set this status!",
                )));
            }

            // If a moderator (non-admin) is completing a review while another moderator holds an
            // active checklist lock, block them from changing the project status.
            if user.role.is_mod()
                && !user.role.is_admin()
                && project_item.inner.status == ProjectStatus::Processing
                && status != &ProjectStatus::Processing
                && let Some(lock) = DBModerationLock::get_with_user(
                    project_item.inner.id,
                    &pool,
                )
                .await
                .wrap_internal_err("fetching moderation lock from database")?
                && lock.moderator_id != db_ids::DBUserId::from(user.id)
                && !lock.expired
            {
                return Err(ApiError::Auth(eyre::eyre!(format!(
                    "This project is currently being moderated by @{}. Please wait for them to finish or for the lock to expire.",
                    lock.moderator_username
                ))));
            }

            if status == &ProjectStatus::Processing {
                sqlx::query!(
                    "
                    UPDATE mods
                    SET moderation_message = NULL, moderation_message_body = NULL, queued = NOW()
                    WHERE (id = $1)
                    ",
                    id as db_ids::DBProjectId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err(
                    "querying database for `project_edit_internal`",
                )?;
            }

            if status.is_approved() && !project_item.inner.status.is_approved()
            {
                sqlx::query!(
                    "
                    UPDATE mods
                    SET approved = NOW()
                    WHERE id = $1 AND approved IS NULL
                    ",
                    id as db_ids::DBProjectId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err(
                    "querying database for `project_edit_internal`",
                )?;
            }

            if status.is_searchable()
                && !project_item.inner.webhook_sent
                && !ENV.PUBLIC_DISCORD_WEBHOOK.is_empty()
                && project_item.inner.components.minecraft_server.is_none()
            {
                crate::util::webhook::send_discord_webhook(
                    project_item.inner.id.into(),
                    &pool,
                    &redis,
                    &ENV.PUBLIC_DISCORD_WEBHOOK,
                    None,
                )
                .await
                .ok();

                sqlx::query!(
                    "
                    UPDATE mods
                    SET webhook_sent = TRUE
                    WHERE id = $1
                    ",
                    id as db_ids::DBProjectId,
                )
                .execute(&mut transaction)
                .await
                .wrap_internal_err(
                    "querying database for `project_edit_internal`",
                )?;
            }

            if user.role.is_mod() && !ENV.MODERATION_SLACK_WEBHOOK.is_empty() {
                crate::util::webhook::send_slack_project_webhook(
                    project_item.inner.id.into(),
                    &pool,
                    &redis,
                    &ENV.MODERATION_SLACK_WEBHOOK,
                    Some(
                        format!(
                            "*<{}/user/{}|{}>* changed project status from *{}* to *{}*",
                            ENV.SITE_URL,
                            user.username,
                            user.username,
                            &project_item.inner.status.as_friendly_str(),
                            status.as_friendly_str(),
                        )
                        .to_string(),
                    ),
                )
                .await
                .ok();
            }

            if team_member.as_ref().is_none_or(|x| !x.accepted) {
                let notified_members = sqlx::query!(
                    "
                    SELECT tm.user_id id
                    FROM team_members tm
                    WHERE tm.team_id = $1 AND tm.accepted
                    ",
                    project_item.inner.team_id as db_ids::DBTeamId
                )
                .fetch(&mut transaction)
                .map_ok(|c| db_models::DBUserId(c.id))
                .try_collect::<Vec<_>>()
                .await
                .wrap_internal_err("fetching notified members from database")?;

                NotificationBuilder {
                    body: NotificationBody::StatusChange {
                        project_id: project_item.inner.id.into(),
                        old_status: project_item.inner.status,
                        new_status: *status,
                    },
                }
                .insert_many(notified_members.clone(), &mut transaction, &redis)
                .await
                .wrap_internal_err(
                    "inserting database records for `project_edit_internal`",
                )?;

                NotificationBuilder {
                    body: if status.is_approved() {
                        NotificationBody::ProjectStatusApproved {
                            project_id: project_item.inner.id.into(),
                        }
                    } else {
                        NotificationBody::ProjectStatusNeutral {
                            project_id: project_item.inner.id.into(),
                            old_status: project_item.inner.status,
                            new_status: *status,
                        }
                    },
                }
                .insert_many(notified_members, &mut transaction, &redis)
                .await
                .wrap_internal_err(
                    "inserting database records for `project_edit_internal`",
                )?;
            }

            ThreadMessageBuilder {
                author_id: Some(user.id.into()),
                body: MessageBody::StatusChange {
                    new_status: *status,
                    old_status: project_item.inner.status,
                },
                thread_id: project_item.thread_id,
                hide_identity: user.role.is_mod(),
            }
            .insert(&mut transaction)
            .await
            .wrap_internal_err(
                "inserting database records for `project_edit_internal`",
            )?;

            sqlx::query!(
                "
                UPDATE mods
                SET status = $1
                WHERE (id = $2)
                ",
                status.as_str(),
                id as db_ids::DBProjectId,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err(
                "querying database for `project_edit_internal`",
            )?;

            if sync_archival_disclosure
                && archival_disclosure.is_some_and(|disclosure| {
                    user.role.is_mod()
                        || disclosure.lock_status.allows_removal()
                })
            {
                db_models::DBProjectDisclosure::remove(
                    project_item.inner.id,
                    ProjectDisclosureType::Archived,
                    user.id.into(),
                    user.role.is_mod(),
                    &mut transaction,
                )
                .await
                .wrap_internal_err("failed to remove archival disclosure")?;
            }
        }
    }

    if let Some(requested_status) = &new_project.requested_status {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the requested status of this project!",
            )));
        }

        if !requested_status
            .map(|x| x.can_be_requested())
            .unwrap_or(true)
        {
            return Err(ApiError::Request(eyre::eyre!(String::from(
                "Specified status cannot be requested!",
            ))));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET requested_status = $1
            WHERE (id = $2)
            ",
            requested_status.map(|x| x.as_str()),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if perms.contains(ProjectPermissions::EDIT_DETAILS) {
        if new_project.categories.is_some() {
            sqlx::query!(
                "
                DELETE FROM mods_categories
                WHERE joining_mod_id = $1 AND is_additional = FALSE
                ",
                id as db_ids::DBProjectId,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err(
                "updating database records for `project_edit_internal`",
            )?;
        }

        if new_project.additional_categories.is_some() {
            sqlx::query!(
                "
                DELETE FROM mods_categories
                WHERE joining_mod_id = $1 AND is_additional = TRUE
                ",
                id as db_ids::DBProjectId,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err(
                "querying database for `project_edit_internal`",
            )?;
        }
    }

    if let Some(categories) = &new_project.categories {
        edit_project_categories(
            categories,
            &perms,
            id as db_ids::DBProjectId,
            false,
            &mut transaction,
        )
        .await
        .wrap_api_err("executing `edit_project_categories`")?;
    }

    if let Some(categories) = &new_project.additional_categories {
        edit_project_categories(
            categories,
            &perms,
            id as db_ids::DBProjectId,
            true,
            &mut transaction,
        )
        .await
        .wrap_api_err("executing `edit_project_categories`")?;
    }

    if let Some(license_url) = &new_project.license_url {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the license URL of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET license_url = $1
            WHERE (id = $2)
            ",
            license_url.as_deref(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(slug) = &new_project.slug {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the slug of this project!",
            )));
        }

        let existing = db_models::DBProject::get(
            &slug.to_lowercase(),
            &mut transaction,
            &redis,
        )
        .await
        .wrap_api_err("checking project slug availability")?;
        if existing.is_some() {
            return Err(ApiError::Request(eyre::eyre!(
                "Slug collides with other project's id!",
            )));
        }

        // Make sure the new slug is different from the old one
        // We are able to unwrap here because the slug is always set
        if !slug.eq(&project_item.inner.slug.clone().unwrap_or_default()) {
            let results = sqlx::query!(
                "
                SELECT EXISTS(
                    SELECT 1 FROM mods
                    WHERE
                        slug = LOWER($1)
                        OR text_id_lower = LOWER($1)
                )
                ",
                slug
            )
            .fetch_one(&mut transaction)
            .await
            .wrap_internal_err(
                "querying database for `project_edit_internal`",
            )?;

            if results.exists.unwrap_or(true) {
                return Err(ApiError::Request(eyre::eyre!(
                    "Slug collides with other project's id!",
                )));
            }
        }

        sqlx::query!(
            "
            UPDATE mods
            SET slug = LOWER($1)
            WHERE (id = $2)
            ",
            Some(slug),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(license) = &new_project.license_id {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the license of this project!",
            )));
        }

        let mut license = license.clone();

        if license.to_lowercase() == "arr" {
            license = models::projects::DEFAULT_LICENSE_ID.to_string();
        }

        spdx::Expression::parse(&license)
            .map_err(|err| eyre::eyre!(err))
            .wrap_request_err("parsing request value")?;

        sqlx::query!(
            "
            UPDATE mods
            SET license = $1
            WHERE (id = $2)
            ",
            license,
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(links) = &new_project.link_urls
        && !links.is_empty()
    {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the links of this project!",
            )));
        }

        let ids_to_delete = links.keys().cloned().collect::<Vec<String>>();
        // Deletes all links from hashmap- either will be deleted or be replaced
        sqlx::query!(
            "
                DELETE FROM mods_links
                WHERE joining_mod_id = $1 AND joining_platform_id IN (
                    SELECT id FROM link_platforms WHERE name = ANY($2)
                )
                ",
            id as db_ids::DBProjectId,
            &ids_to_delete
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;

        for (platform, url) in links {
            if let Some(url) = url {
                let platform_id = db_models::categories::LinkPlatform::get_id(
                    platform,
                    &mut transaction,
                )
                .await
                .wrap_internal_err("fetching link platform from database")?
                .wrap_request_err_with(|| {
                    format!("platform `{}` does not exist", platform.clone())
                })?;
                sqlx::query!(
                        "
                        INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
                        VALUES ($1, $2, $3)
                        ",
                        id as db_ids::DBProjectId,
                        platform_id as db_ids::LinkPlatformId,
                        url
                    )
                    .execute(&mut transaction)
                    .await.wrap_internal_err("querying database for `project_edit_internal`")?;
            }
        }
    }
    if let Some(moderation_message) = &new_project.moderation_message {
        if !user.role.is_mod()
            && (!project_item.inner.status.is_approved()
                || moderation_message.is_some())
        {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the moderation message of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET moderation_message = $1
            WHERE (id = $2)
            ",
            moderation_message.as_deref(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(moderation_message_body) = &new_project.moderation_message_body
    {
        if !user.role.is_mod()
            && (!project_item.inner.status.is_approved()
                || moderation_message_body.is_some())
        {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the moderation message body of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET moderation_message_body = $1
            WHERE (id = $2)
            ",
            moderation_message_body.as_deref(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(description) = &new_project.description {
        if !perms.contains(ProjectPermissions::EDIT_BODY) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the description (body) of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET description = $1
            WHERE (id = $2)
            ",
            description,
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(monetization_status) = &new_project.monetization_status {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the monetization status of this project!",
            )));
        }

        if (*monetization_status == MonetizationStatus::ForceDemonetized
            || project_item.inner.monetization_status
                == MonetizationStatus::ForceDemonetized)
            && !user.role.is_mod()
        {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the monetization status of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET monetization_status = $1
            WHERE (id = $2)
            ",
            monetization_status.as_str(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if let Some(side_types_migration_review_status) =
        &new_project.side_types_migration_review_status
    {
        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the side types migration review status of this project!",
            )));
        }

        sqlx::query!(
            "
            UPDATE mods
            SET side_types_migration_review_status = $1
            WHERE id = $2
            ",
            side_types_migration_review_status.as_str(),
            id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("querying database for `project_edit_internal`")?;
    }

    if !new_project.loader_fields.is_empty() {
        for version in db_models::DBVersion::get_many(
            &project_item.versions,
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching versions from database")?
        {
            match super::versions::version_edit_helper(
                req.clone(),
                (VersionId::from(version.inner.id),),
                pool.clone(),
                redis.clone(),
                super::versions::EditVersion {
                    fields: new_project.loader_fields.clone(),
                    ..Default::default()
                },
                session_queue.clone(),
                search_state.clone(),
            )
            .await
            {
                // An `InvalidInput` error being returned from this route when only
                // editing the loader fields means that such fields are not valid for
                // the loaders defined for this version, which is a common case for
                // projects with heterogeneous loaders across versions and is best
                // handled with opportunistic update semantics
                Ok(_) | Err(ApiError::Request(_)) => continue,
                err => return err,
            }
        }
    }

    // components

    async fn update<E: exp::component::ComponentEdit>(
        _txn: &mut PgTransaction<'_>,
        _project_id: DBProjectId,
        edit: Option<Option<E>>,
        mut component: &mut Option<E::Component>,
        perms: ProjectPermissions,
    ) -> Result<bool, ApiError> {
        let Some(edit) = edit else {
            // component is not specified in the input JSON - leave alone
            return Ok(false);
        };

        if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You do not have the permissions to edit the components of this project!",
            )));
        }

        match (&mut component, edit) {
            (None, None) => {}
            (Some(_), None) => {
                // component is `null` in the input JSON - remove component
                *component = None;
            }
            (None, Some(edit)) => {
                // component is specified in the JSON and is non-null - create new component
                *component =
                    Some(edit.create().wrap_request_err_with(|| {
                        eyre!(
                            "failed to create `{}` component",
                            type_name::<E::Component>()
                        )
                    })?);
            }
            (Some(component), Some(edit)) => {
                // edit component
                edit.apply_to(component).await.wrap_internal_err_with(
                    || {
                        eyre!(
                            "failed to update `{}` component",
                            type_name::<E::Component>()
                        )
                    },
                )?;
            }
        }

        Ok(true)
    }

    let mut reindex_versions = new_project.categories.is_some()
        || new_project.additional_categories.is_some();
    let became_searchable = !project_item.inner.status.is_searchable()
        && new_project
            .status
            .is_some_and(|status| status.is_searchable());
    let became_unsearchable = project_item.inner.status.is_searchable()
        && new_project
            .status
            .is_some_and(|status| !status.is_searchable());

    reindex_versions |= update(
        &mut transaction,
        id,
        new_project.minecraft_server,
        &mut project_item.inner.components.minecraft_server,
        perms,
    )
    .await
    .wrap_api_err("updating Minecraft server component")?;
    reindex_versions |= update(
        &mut transaction,
        id,
        new_project.minecraft_java_server,
        &mut project_item.inner.components.minecraft_java_server,
        perms,
    )
    .await
    .wrap_api_err("updating Minecraft Java server component")?;
    reindex_versions |= update(
        &mut transaction,
        id,
        new_project.minecraft_bedrock_server,
        &mut project_item.inner.components.minecraft_bedrock_server,
        perms,
    )
    .await
    .wrap_api_err("updating Minecraft Bedrock server component")?;

    let components_serial = project_item.inner.components.clone();

    exp::component::kinds_valid(
        &components_serial.component_kinds(),
        &exp::PROJECT_COMPONENT_RELATIONS,
    )
    .wrap_request_err("invalid component kinds")?;

    sqlx::query!(
        "
        UPDATE mods
        SET components = $1
        WHERE id = $2
        ",
        serde_json::to_value(&components_serial)
            .expect("serialization shouldn't fail"),
        id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to update components")?;

    let reloaded_project = if validate_for_review {
        validate::ensure_project_is_valid_for_review(
            id,
            &pool,
            &mut transaction,
            &redis,
        )
        .await?
    } else {
        project_item.clone()
    };

    // check new description and body for links to associated images
    // if they no longer exist in the description or body, delete them
    let checkable_strings: Vec<&str> =
        vec![&new_project.description, &new_project.summary]
            .into_iter()
            .filter_map(|x| x.as_ref().map(|y| y.as_str()))
            .collect();

    let context = ImageContext::Project {
        project_id: Some(id.into()),
    };

    img::delete_unused_images(
        context,
        checkable_strings,
        &mut transaction,
        &redis,
    )
    .await
    .wrap_api_err("deleting unused images")?;

    if submit_for_review {
        submit_project_for_review(
            &reloaded_project,
            &user,
            team_member.as_ref().is_none_or(|member| !member.accepted),
            sync_archival_disclosure,
            &mut transaction,
            &redis,
        )
        .await?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    if became_unsearchable {
        db_models::DBProject::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
        search_state
            .queue
            .push_project_removal(project_item.inner.id.into())
            .await;
    } else if reindex_versions || became_searchable {
        db_models::DBProject::clear_cache(
            project_item.inner.id,
            project_item.inner.slug,
            None,
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
        search_state
            .queue
            .push_project_with_all_versions_change(project_item.inner.id.into())
            .await;
    } else {
        clear_project_cache_and_queue_search(
            &redis,
            &search_state,
            project_item.inner.id,
            project_item.inner.slug,
            None,
        )
        .await
        .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;
    }

    Ok(HttpResponse::NoContent().body(""))
}

async fn submit_project_for_review(
    project: &ProjectQueryResult,
    user: &User,
    notify_team_members: bool,
    sync_archival_disclosure: bool,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<(), ApiError> {
    let archival_disclosure =
        db_models::DBProjectDisclosure::get_many_for_project(
            project.inner.id,
            false,
            &mut *transaction,
        )
        .await
        .wrap_internal_err("failed to fetch project disclosures")?
        .into_iter()
        .find(|disclosure| {
            matches!(disclosure.disclosure, ProjectDisclosure::Archived { .. })
        });

    sqlx::query!(
        "
                    UPDATE mods
                    SET moderation_message = NULL, moderation_message_body = NULL, queued = NOW()
                    WHERE (id = $1)
                    ",
        project.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut *transaction)
    .await
    .wrap_internal_err("querying database for `project_edit_internal`")?;

    if notify_team_members {
        let notified_members = sqlx::query!(
            "
                    SELECT tm.user_id id
                    FROM team_members tm
                    WHERE tm.team_id = $1 AND tm.accepted
                    ",
            project.inner.team_id as db_ids::DBTeamId
        )
        .fetch(&mut *transaction)
        .map_ok(|row| db_models::DBUserId(row.id))
        .try_collect::<Vec<_>>()
        .await
        .wrap_internal_err("fetching notified members from database")?;

        NotificationBuilder {
            body: NotificationBody::StatusChange {
                project_id: project.inner.id.into(),
                old_status: project.inner.status,
                new_status: ProjectStatus::Processing,
            },
        }
        .insert_many(notified_members.clone(), &mut *transaction, redis)
        .await
        .wrap_internal_err(
            "inserting database records for `project_edit_internal`",
        )?;

        NotificationBuilder {
            body: NotificationBody::ProjectStatusNeutral {
                project_id: project.inner.id.into(),
                old_status: project.inner.status,
                new_status: ProjectStatus::Processing,
            },
        }
        .insert_many(notified_members, &mut *transaction, redis)
        .await
        .wrap_internal_err(
            "inserting database records for `project_edit_internal`",
        )?;
    }

    ThreadMessageBuilder {
        author_id: Some(user.id.into()),
        body: MessageBody::StatusChange {
            new_status: ProjectStatus::Processing,
            old_status: project.inner.status,
        },
        thread_id: project.thread_id,
        hide_identity: false,
    }
    .insert(&mut *transaction)
    .await
    .wrap_internal_err(
        "inserting database records for `project_edit_internal`",
    )?;

    sqlx::query!(
        "
                UPDATE mods
                SET status = $1
                WHERE (id = $2)
                ",
        ProjectStatus::Processing.as_str(),
        project.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut *transaction)
    .await
    .wrap_internal_err("querying database for `project_edit_internal`")?;

    if sync_archival_disclosure
        && archival_disclosure
            .is_some_and(|disclosure| disclosure.lock_status.allows_removal())
    {
        db_models::DBProjectDisclosure::remove(
            project.inner.id,
            ProjectDisclosureType::Archived,
            user.id.into(),
            false,
            &mut *transaction,
        )
        .await
        .wrap_internal_err("failed to remove archival disclosure")?;
    }

    Ok(())
}

pub async fn edit_project_categories(
    categories: &Vec<String>,
    perms: &ProjectPermissions,
    project_id: db_ids::DBProjectId,
    is_additional: bool,
    transaction: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
    if !perms.contains(ProjectPermissions::EDIT_DETAILS) {
        let additional_str = if is_additional { "additional " } else { "" };
        return Err(ApiError::Auth(eyre::eyre!(format!(
            "You do not have the permissions to edit the {additional_str}categories of this project!"
        ))));
    }

    let mut mod_categories = Vec::new();
    for category in categories {
        let category_ids = db_models::categories::Category::get_ids(
            category,
            &mut *transaction,
        )
        .await
        .wrap_internal_err("fetching category from Redis")?;
        // TODO: We should filter out categories that don't match the project type of any of the versions
        // ie: if mod and modpack both share a name this should only have modpack if it only has a modpack as a version

        let mcategories = category_ids
            .values()
            .map(|&category_id| DBModCategory {
                project_id,
                category_id,
                is_additional,
            })
            .collect::<Vec<_>>();
        mod_categories.extend(mcategories);
    }
    DBModCategory::insert_many(mod_categories, &mut *transaction)
        .await
        .wrap_internal_err("inserting mod categories into database")?;

    Ok(())
}

// TODO: Re-add this if we want to match v3 Projects structure to v3 Search Result structure, otherwise, delete
// #[derive(Serialize, Deserialize)]
// pub struct ReturnSearchResults {
//     pub hits: Vec<Project>,
//     pub page: usize,
//     pub hits_per_page: usize,
//     pub total_hits: usize,
// }

/// Search projects.
#[utoipa::path(
	tag = "search",
    get,
    operation_id = "v3SearchProjects",
    params(
        ("query" = Option<String>, Query, description = "The query to search for"),
        ("facets" = Option<String>, Query, description = "Search facets JSON"),
        ("filters" = Option<String>, Query, description = "Search filters JSON"),
        ("new_filters" = Option<String>, Query, description = "Search filters JSON"),
        ("index" = Option<String>, Query, description = "Search index to use"),
        ("offset" = Option<String>, Query, description = "Search result offset"),
        ("limit" = Option<String>, Query, description = "Maximum number of search results"),
        ("version" = Option<String>, Query, description = "Game version to filter for")
    ),
    responses(
        (status = 200, description = "Expected response to a valid request", body = SearchResults),
        (status = 400, description = "Request was invalid, see given error")
    )
)]
#[get("/search")]
pub async fn project_search(
    web::Query(info): web::Query<SearchQuery>,
    search_backend: web::Data<dyn SearchBackend>,
    redis: web::Data<RedisPool>,
) -> Result<web::Json<SearchResults>, ApiError> {
    let results = search_backend
        .search_for_project(&SearchRequest::from(info), &redis)
        .await
        .wrap_api_err("searching projects")?;

    // TODO: add this back
    // let results = ReturnSearchResults {
    //     hits: results
    //         .hits
    //         .into_iter()
    //         .filter_map(Project::from_search)
    //         .collect::<Vec<_>>(),
    //     page: results.page,
    //     hits_per_page: results.hits_per_page,
    //     total_hits: results.total_hits,
    // };

    Ok(web::Json(results))
}

// for more complicated search queries
/// Search projects.
#[utoipa::path(
	tag = "search",
	request_body = serde_json::Value,
	responses((status = OK, body = SearchResults))
)]
#[post("/search")]
pub async fn project_search_post(
    web::Json(info): web::Json<SearchRequest>,
    search_backend: web::Data<dyn SearchBackend>,
    redis: web::Data<RedisPool>,
) -> Result<web::Json<SearchResults>, ApiError> {
    let results = search_backend
        .search_for_project(&info, &redis)
        .await
        .wrap_api_err("searching projects")?;
    Ok(web::Json(results))
}

//checks the validity of a project id or slug
/// Check project availability.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = OK, body = ProjectCheckResponse))
)]
#[get("/{id}/check")]
pub async fn project_get_check(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    project_get_check_internal(info, pool, redis).await
}

pub async fn project_get_check_internal(
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
) -> Result<HttpResponse, ApiError> {
    let slug = info.into_inner().0;

    let project_data = db_models::DBProject::get(&slug, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?;

    if let Some(project) = project_data {
        Ok(HttpResponse::Ok().json(ProjectCheckResponse {
            id: models::ids::ProjectId::from(project.inner.id),
        }))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct DependencyInfo {
    pub projects: Vec<Project>,
    pub versions: Vec<models::projects::Version>,
}

/// List project dependencies.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = OK, body = DependencyInfo))
)]
#[get("/{project_id}/dependencies")]
pub async fn dependency_list(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    dependency_list_internal(req, info, pool, ro_pool, redis, session_queue)
        .await
}

pub async fn dependency_list_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let string = info.into_inner().0;

    let result = db_models::DBProject::get(&string, &***ro_pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?;

    let user_option = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|x| x.1)
    .ok();

    if let Some(project) = result {
        if !is_visible_project(&project.inner, &user_option, &pool, false)
            .await
            .wrap_api_err("checking project visibility")?
        {
            return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
        }

        let dependencies = database::DBProject::get_dependencies(
            project.inner.id,
            &***ro_pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching projects from database")?;
        let project_ids = dependencies
            .iter()
            .filter_map(|x| {
                if x.0.is_none() {
                    if let Some(mod_dependency_id) = x.2 {
                        Some(mod_dependency_id)
                    } else {
                        x.1
                    }
                } else {
                    x.1
                }
            })
            .unique()
            .collect::<Vec<_>>();

        let dep_version_ids = dependencies
            .iter()
            .filter_map(|x| x.0)
            .unique()
            .collect::<Vec<db_models::DBVersionId>>();
        let (projects_result, versions_result) = futures::future::try_join(
            database::DBProject::get_many_ids(
                &project_ids,
                &***ro_pool,
                &redis,
            ),
            async {
                database::DBVersion::get_many(
                    &dep_version_ids,
                    &***ro_pool,
                    &redis,
                )
                .await
                .wrap_internal_err("failed to fetch dependency versions")
            },
        )
        .await
        .wrap_api_err("fetching project dependencies")?;

        let mut projects = filter_visible_projects(
            projects_result,
            &user_option,
            &pool,
            false,
        )
        .await
        .wrap_api_err("filtering visible projects")?;
        let mut versions = filter_visible_versions(
            versions_result,
            &user_option,
            &pool,
            &ro_pool,
            &redis,
        )
        .await
        .wrap_api_err("filtering visible versions")?;

        projects.sort_by_key(|b| Reverse(b.published));
        projects.dedup_by(|a, b| a.id == b.id);

        versions.sort_by_key(|b| Reverse(b.date_published));
        versions.dedup_by(|a, b| a.id == b.id);

        Ok(HttpResponse::Ok().json(DependencyInfo { projects, versions }))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

pub struct CategoryChanges<'a> {
    pub categories: &'a Option<Vec<String>>,
    pub add_categories: &'a Option<Vec<String>>,
    pub remove_categories: &'a Option<Vec<String>>,
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

    #[validate(custom(
        function = " crate::util::validate::validate_url_hashmap_optional_values"
    ))]
    pub link_urls: Option<HashMap<String, Option<String>>>,
}

#[utoipa::path(
	tag = "projects",
	params(("ids" = String, Query)),
	responses((status = NO_CONTENT))
)]
#[patch("/projects")]
pub async fn projects_edit_route(
    req: HttpRequest,
    ids: web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    bulk_edit_project: web::Json<BulkEditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    projects_edit(
        req,
        ids,
        pool,
        bulk_edit_project,
        redis,
        session_queue,
        search_state,
    )
    .await
}

pub async fn projects_edit(
    req: HttpRequest,
    web::Query(ids): web::Query<ProjectIds>,
    pool: web::Data<PgPool>,
    bulk_edit_project: web::Json<BulkEditProject>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    bulk_edit_project
        .validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let project_ids: Vec<db_ids::DBProjectId> =
        serde_json::from_str::<Vec<ProjectId>>(&ids.ids)
            .wrap_request_err("deserializing JSON data")?
            .into_iter()
            .map(|x| x.into())
            .collect();

    let projects_data =
        db_models::DBProject::get_many_ids(&project_ids, &**pool, &redis)
            .await
            .wrap_api_err("fetching projects to edit")?;

    if let Some(id) = project_ids
        .iter()
        .find(|x| !projects_data.iter().any(|y| x == &&y.inner.id))
    {
        return Err(ApiError::Request(eyre::eyre!(format!(
            "Project {} not found",
            ProjectId(id.0 as u64)
        ))));
    }

    let team_ids = projects_data
        .iter()
        .map(|x| x.inner.team_id)
        .collect::<Vec<db_models::DBTeamId>>();
    let team_members = db_models::DBTeamMember::get_from_team_full_many(
        &team_ids, &**pool, &redis,
    )
    .await
    .wrap_internal_err("fetching team members from database")?;

    let organization_ids = projects_data
        .iter()
        .filter_map(|x| x.inner.organization_id)
        .collect::<Vec<db_models::DBOrganizationId>>();
    let organizations = db_models::DBOrganization::get_many_ids(
        &organization_ids,
        &**pool,
        &redis,
    )
    .await
    .wrap_internal_err("fetching organizations from database")?;

    let organization_team_ids = organizations
        .iter()
        .map(|x| x.team_id)
        .collect::<Vec<db_models::DBTeamId>>();
    let organization_team_members =
        db_models::DBTeamMember::get_from_team_full_many(
            &organization_team_ids,
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching team members from database")?;

    let categories = db_models::categories::Category::list(&**pool, &redis)
        .await
        .wrap_internal_err("fetching category from Redis")?;
    let link_platforms =
        db_models::categories::LinkPlatform::list(&**pool, &redis)
            .await
            .wrap_internal_err("fetching link platform from Redis")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;
    let mut changed_projects = Vec::new();

    for project in projects_data {
        if !user.role.is_mod() {
            let team_member = team_members.iter().find(|x| {
                x.team_id == project.inner.team_id
                    && x.user_id == user.id.into()
            });

            let organization = project
                .inner
                .organization_id
                .and_then(|oid| organizations.iter().find(|x| x.id == oid));

            let organization_team_member =
                if let Some(organization) = organization {
                    organization_team_members.iter().find(|x| {
                        x.team_id == organization.team_id
                            && x.user_id == user.id.into()
                    })
                } else {
                    None
                };

            let permissions = ProjectPermissions::get_permissions_by_role(
                &user.role,
                &team_member.cloned(),
                &organization_team_member.cloned(),
            )
            .unwrap_or_default();

            if team_member.is_some() {
                if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
                    return Err(ApiError::Auth(eyre::eyre!(format!(
                        "You do not have the permissions to bulk edit project {}!",
                        project.inner.name
                    ))));
                }
            } else if project.inner.status.is_hidden() {
                return Err(ApiError::Request(eyre::eyre!(format!(
                    "Project {} not found",
                    ProjectId(project.inner.id.0 as u64)
                ))));
            } else {
                return Err(ApiError::Auth(eyre::eyre!(format!(
                    "You are not a member of project {}!",
                    project.inner.name
                ))));
            };
        }

        if let Some(links) = &bulk_edit_project.link_urls {
            let mut candidate = Project::from(project.clone());
            validate::apply_link_changes(&mut candidate, links);
            let nags = crate::validate::project::validate_link_fields(
                &candidate,
                crate::validate::project::LinkValidationScope {
                    external: true,
                    ..Default::default()
                },
            );
            validate::require_valid_project(nags)?;
        }
        let mut reindex_versions = bulk_edit_project_categories(
            &categories,
            &project.categories,
            project.inner.id as db_ids::DBProjectId,
            CategoryChanges {
                categories: &bulk_edit_project.categories,
                add_categories: &bulk_edit_project.add_categories,
                remove_categories: &bulk_edit_project.remove_categories,
            },
            3,
            false,
            &mut transaction,
        )
        .await
        .wrap_api_err("executing `bulk_edit_project_categories`")?;

        reindex_versions |= bulk_edit_project_categories(
            &categories,
            &project.additional_categories,
            project.inner.id as db_ids::DBProjectId,
            CategoryChanges {
                categories: &bulk_edit_project.additional_categories,
                add_categories: &bulk_edit_project.add_additional_categories,
                remove_categories: &bulk_edit_project
                    .remove_additional_categories,
            },
            256,
            true,
            &mut transaction,
        )
        .await
        .wrap_api_err("executing `bulk_edit_project_categories`")?;

        if let Some(links) = &bulk_edit_project.link_urls {
            let ids_to_delete = links.keys().cloned().collect::<Vec<String>>();
            // Deletes all links from hashmap- either will be deleted or be replaced
            sqlx::query!(
                "
                DELETE FROM mods_links
                WHERE joining_mod_id = $1 AND joining_platform_id IN (
                    SELECT id FROM link_platforms WHERE name = ANY($2)
                )
                ",
                project.inner.id as db_ids::DBProjectId,
                &ids_to_delete
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("querying database for `projects_edit`")?;

            for (platform, url) in links {
                if let Some(url) = url {
                    let platform_id = link_platforms
                        .iter()
                        .find(|x| &x.name == platform)
                        .wrap_request_err_with(|| {
                            format!(
                                "platform `{}` does not exist",
                                platform.clone()
                            )
                        })?
                        .id;
                    sqlx::query!(
                        "
                        INSERT INTO mods_links (joining_mod_id, joining_platform_id, url)
                        VALUES ($1, $2, $3)
                        ",
                        project.inner.id as db_ids::DBProjectId,
                        platform_id as db_ids::LinkPlatformId,
                        url
                    )
                    .execute(&mut transaction)
                    .await.wrap_internal_err("querying database for `projects_edit`")?;
                }
            }
        }

        if project.inner.status == ProjectStatus::Processing {
            validate::ensure_project_is_valid_for_review(
                project.inner.id,
                &pool,
                &mut transaction,
                &redis,
            )
            .await?;
        }

        changed_projects.push((
            project.inner.id,
            project.inner.slug,
            reindex_versions,
        ));
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    for (project_id, slug, reindex_versions) in changed_projects {
        if reindex_versions {
            db_models::DBProject::clear_cache(project_id, slug, None, &redis)
                .await
                .wrap_internal_err("clearing cached data from Redis")?;
            search_state
                .queue
                .push_project_with_all_versions_change(project_id.into())
                .await;
        } else {
            clear_project_cache_and_queue_search(
                &redis,
                &search_state,
                project_id,
                slug,
                None,
            )
            .await
            .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;
        }
    }

    Ok(HttpResponse::NoContent().body(""))
}

pub async fn bulk_edit_project_categories(
    all_db_categories: &[db_models::categories::Category],
    project_categories: &Vec<String>,
    project_id: db_ids::DBProjectId,
    bulk_changes: CategoryChanges<'_>,
    max_num_categories: usize,
    is_additional: bool,
    transaction: &mut PgTransaction<'_>,
) -> Result<bool, ApiError> {
    let mut set_categories =
        if let Some(categories) = bulk_changes.categories.clone() {
            categories
        } else {
            project_categories.clone()
        };

    if let Some(delete_categories) = &bulk_changes.remove_categories {
        for category in delete_categories {
            if let Some(pos) = set_categories.iter().position(|x| x == category)
            {
                set_categories.remove(pos);
            }
        }
    }

    if let Some(add_categories) = &bulk_changes.add_categories {
        for category in add_categories {
            if set_categories.len() < max_num_categories {
                set_categories.push(category.clone());
            } else {
                break;
            }
        }
    }

    let changed = &set_categories != project_categories;
    if changed {
        sqlx::query!(
            "
            DELETE FROM mods_categories
            WHERE joining_mod_id = $1 AND is_additional = $2
            ",
            project_id as db_ids::DBProjectId,
            is_additional
        )
        .execute(&mut *transaction)
        .await
        .wrap_internal_err(
            "querying database for `bulk_edit_project_categories`",
        )?;

        let mut mod_categories = Vec::new();
        for category in set_categories {
            let category_id = all_db_categories
                .iter()
                .find(|x| x.category == category)
                .wrap_request_err_with(|| {
                    format!("category `{}` does not exist", category.clone())
                })?
                .id;
            mod_categories.push(DBModCategory {
                project_id,
                category_id,
                is_additional,
            });
        }
        DBModCategory::insert_many(mod_categories, &mut *transaction)
            .await
            .wrap_internal_err("inserting mod categories into database")?;
    }

    Ok(changed)
}

#[derive(Serialize, Deserialize)]
pub struct Extension {
    pub ext: String,
}

#[allow(clippy::too_many_arguments)]
/// Update a project icon.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	params(
		("ext" = String, Query)
	),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = NO_CONTENT))
)]
#[patch("/{id}/icon")]
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
    project_icon_edit_internal(
        web::Query(ext),
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
}

pub async fn project_icon_edit_internal(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let string = info.into_inner().0;

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("fetching team member from database")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre::eyre!(
                "The specified project does not exist!",
            )));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this project's icon.",
            )));
        }
    }

    delete_old_images(
        project_item.inner.icon_url,
        project_item.inner.raw_icon_url,
        FileHostPublicity::Public,
        &**file_host,
    )
    .await
    .wrap_api_err("deleting old images")?;

    let bytes = read_limited_from_payload(
        &mut payload,
        262144,
        "Icons must be smaller than 256KiB",
    )
    .await
    .wrap_api_err("executing `read_limited_from_payload`")?;

    let project_id: ProjectId = project_item.inner.id.into();
    let upload_result = upload_image_optimized(
        &format!("data/{project_id}"),
        FileHostPublicity::Public,
        bytes.freeze(),
        &ext.ext,
        Some(96),
        Some(1.0),
        &**file_host,
    )
    .await
    .wrap_api_err("uploading image")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    sqlx::query!(
        "
            UPDATE mods
            SET icon_url = $1, raw_icon_url = $2, color = $3
            WHERE (id = $4)
            ",
        upload_result.url,
        upload_result.raw_url,
        upload_result.color.map(|x| x as i32),
        project_item.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("querying database for `project_icon_edit_internal`")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;
    clear_project_cache_and_queue_search(
        &redis,
        &search_state,
        project_item.inner.id,
        project_item.inner.slug,
        None,
    )
    .await
    .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Delete a project icon.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = NO_CONTENT))
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
    delete_project_icon_internal(
        req,
        info,
        pool,
        redis,
        file_host,
        session_queue,
        search_state,
    )
    .await
}

pub async fn delete_project_icon_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let string = info.into_inner().0;

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("fetching team member from database")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre::eyre!(
                "The specified project does not exist!",
            )));
        }
        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this project's icon.",
            )));
        }
    }

    delete_old_images(
        project_item.inner.icon_url,
        project_item.inner.raw_icon_url,
        FileHostPublicity::Public,
        &**file_host,
    )
    .await
    .wrap_api_err("deleting old images")?;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    sqlx::query!(
        "
        UPDATE mods
        SET icon_url = NULL, raw_icon_url = NULL, color = NULL
        WHERE (id = $1)
        ",
        project_item.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err(
        "querying database for `delete_project_icon_internal`",
    )?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;
    clear_project_cache_and_queue_search(
        &redis,
        &search_state,
        project_item.inner.id,
        project_item.inner.slug,
        None,
    )
    .await
    .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Validate)]
pub struct GalleryCreateQuery {
    pub featured: bool,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<String>,
    pub ordering: Option<i64>,
}

#[allow(clippy::too_many_arguments)]
/// Add a gallery item.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	params(
		("ext" = String, Query),
		("featured" = bool, Query),
		("name" = Option<String>, Query),
		("description" = Option<String>, Query),
		("ordering" = Option<i64>, Query)
	),
	request_body(content = Vec<u8>, content_type = "application/octet-stream"),
	responses((status = NO_CONTENT))
)]
#[post("/{id}/gallery")]
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
    add_gallery_item_internal(
        web::Query(ext),
        req,
        web::Query(item),
        info,
        pool,
        redis,
        file_host,
        payload,
        session_queue,
        search_state,
    )
    .await
}

pub async fn add_gallery_item_internal(
    web::Query(ext): web::Query<Extension>,
    req: HttpRequest,
    web::Query(item): web::Query<GalleryCreateQuery>,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    mut payload: web::Payload,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    item.validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let string = info.into_inner().0;

    let project_item = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    if project_item.gallery_items.len() > 64 {
        return Err(ApiError::Auth(eyre::eyre!(
            "You have reached the maximum of gallery images to upload.",
        )));
    }

    if !user.role.is_admin() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("fetching team member from database")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre::eyre!(
                "The specified project does not exist!",
            )));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this project's gallery.",
            )));
        }
    }

    let bytes = read_limited_from_payload(
        &mut payload,
        5 * (1 << 20),
        "Gallery image exceeds the maximum of 5MiB.",
    )
    .await
    .wrap_api_err("executing `read_limited_from_payload`")?;

    let id: ProjectId = project_item.inner.id.into();
    let upload_result = upload_image_optimized(
        &format!("data/{id}/images"),
        FileHostPublicity::Public,
        bytes.freeze(),
        &ext.ext,
        Some(350),
        Some(1.0),
        &**file_host,
    )
    .await
    .wrap_api_err("uploading image")?;

    if project_item
        .gallery_items
        .iter()
        .any(|x| x.image_url == upload_result.url)
    {
        return Err(ApiError::Request(eyre::eyre!(
            "You may not upload duplicate gallery images!",
        )));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    if item.featured {
        sqlx::query!(
            "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
            project_item.inner.id as db_ids::DBProjectId,
            false,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "querying database for `add_gallery_item_internal`",
        )?;
    }

    let gallery_item = vec![db_models::project_item::DBGalleryItem {
        image_url: upload_result.url.clone(),
        raw_image_url: upload_result.raw_url.clone(),
        featured: item.featured,
        name: item.name,
        description: item.description,
        created: Utc::now(),
        ordering: item.ordering.unwrap_or(0),
    }];
    DBGalleryItem::insert_many(
        gallery_item,
        project_item.inner.id,
        &mut transaction,
    )
    .await
    .wrap_internal_err("inserting galleries into database")?;

    let validation_error =
        if project_item.inner.status == ProjectStatus::Processing {
            validate::ensure_project_is_valid_for_review(
                project_item.inner.id,
                &pool,
                &mut transaction,
                &redis,
            )
            .await
            .err()
        } else {
            None
        };
    if let Some(error) = validation_error {
        delete_old_images(
            Some(upload_result.url),
            Some(upload_result.raw_url),
            FileHostPublicity::Public,
            &**file_host,
        )
        .await
        .wrap_api_err("deleting rejected gallery image upload")?;
        return Err(error);
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;
    clear_project_cache_and_queue_search(
        &redis,
        &search_state,
        project_item.inner.id,
        project_item.inner.slug,
        None,
    )
    .await
    .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize, Validate)]
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
    pub name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    #[validate(length(min = 1, max = 2048))]
    pub description: Option<Option<String>>,
    pub ordering: Option<i64>,
}

/// Update a gallery item.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	params(
		("url" = String, Query),
		("featured" = Option<bool>, Query),
		("name" = Option<String>, Query),
		("description" = Option<String>, Query),
		("ordering" = Option<i64>, Query)
	),
	responses((status = NO_CONTENT))
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
    edit_gallery_item_internal(
        req,
        web::Query(item),
        pool,
        redis,
        session_queue,
        search_state,
    )
    .await
}

pub async fn edit_gallery_item_internal(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryEditQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    item.validate()
        .map_err(|err| eyre::eyre!(err))
        .wrap_request_err("validating request")?;

    let result = sqlx::query!(
        "
        SELECT id, mod_id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("querying database for `edit_gallery_item_internal`")?
    .wrap_request_err_with(|| {
        format!(
            "gallery item at URL `{}` is not part of the project's gallery",
            item.url
        )
    })?;

    let project_item = db_models::DBProject::get_id(
        database::models::DBProjectId(result.mod_id),
        &**pool,
        &redis,
    )
    .await
    .wrap_api_err("fetching project from database")?
    .wrap_request_err_with(|| {
        "the specified project does not exist!".to_string()
    })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("fetching team member from database")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre::eyre!(
                "The specified project does not exist!",
            )));
        }
        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this project's gallery.",
            )));
        }
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    if let Some(featured) = item.featured {
        if featured {
            sqlx::query!(
                "
                UPDATE mods_gallery
                SET featured = $2
                WHERE mod_id = $1
                ",
                project_item.inner.id as db_ids::DBProjectId,
                false,
            )
            .execute(&mut transaction)
            .await
            .wrap_internal_err("fetching featured status from database")?;
        }

        sqlx::query!(
            "
            UPDATE mods_gallery
            SET featured = $2
            WHERE id = $1
            ",
            result.id,
            featured
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "querying database for `edit_gallery_item_internal`",
        )?;
    }
    if let Some(name) = item.name {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET name = $2
            WHERE id = $1
            ",
            result.id,
            name
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "querying database for `edit_gallery_item_internal`",
        )?;
    }
    if let Some(description) = item.description {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET description = $2
            WHERE id = $1
            ",
            result.id,
            description
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "querying database for `edit_gallery_item_internal`",
        )?;
    }
    if let Some(ordering) = item.ordering {
        sqlx::query!(
            "
            UPDATE mods_gallery
            SET ordering = $2
            WHERE id = $1
            ",
            result.id,
            ordering
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "querying database for `edit_gallery_item_internal`",
        )?;
    }

    if project_item.inner.status == ProjectStatus::Processing {
        validate::ensure_project_is_valid_for_review(
            project_item.inner.id,
            &pool,
            &mut transaction,
            &redis,
        )
        .await?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    clear_project_cache_and_queue_search(
        &redis,
        &search_state,
        project_item.inner.id,
        project_item.inner.slug,
        None,
    )
    .await
    .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;

    Ok(HttpResponse::NoContent().body(""))
}

#[derive(Serialize, Deserialize)]
pub struct GalleryDeleteQuery {
    pub url: String,
}

/// Delete a gallery item.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	params(
		("url" = String, Query)
	),
	responses((status = NO_CONTENT))
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
    delete_gallery_item_internal(
        req,
        web::Query(item),
        pool,
        redis,
        file_host,
        session_queue,
        search_state,
    )
    .await
}

pub async fn delete_gallery_item_internal(
    req: HttpRequest,
    web::Query(item): web::Query<GalleryDeleteQuery>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    file_host: web::Data<dyn FileHost>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let item = sqlx::query!(
        "
        SELECT id, image_url, raw_image_url, mod_id FROM mods_gallery
        WHERE image_url = $1
        ",
        item.url
    )
    .fetch_optional(&**pool)
    .await
    .wrap_internal_err("querying database for `delete_gallery_item_internal`")?
    .wrap_request_err_with(|| {
        format!(
            "gallery item at URL `{}` is not part of the project's gallery",
            item.url
        )
    })?;

    let project_item = db_models::DBProject::get_id(
        database::models::DBProjectId(item.mod_id),
        &**pool,
        &redis,
    )
    .await
    .wrap_api_err("fetching project from database")?
    .wrap_request_err_with(|| {
        "the specified project does not exist!".to_string()
    })?;

    if !user.role.is_mod() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project_item.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("fetching team member from database")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre::eyre!(
                "The specified project does not exist!",
            )));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::EDIT_DETAILS) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to edit this project's gallery.",
            )));
        }
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    sqlx::query!(
        "
        DELETE FROM mods_gallery
        WHERE id = $1
        ",
        item.id
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err(
        "querying database for `delete_gallery_item_internal`",
    )?;

    if project_item.inner.status == ProjectStatus::Processing {
        validate::ensure_project_is_valid_for_review(
            project_item.inner.id,
            &pool,
            &mut transaction,
            &redis,
        )
        .await?;
    }

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    delete_old_images(
        Some(item.image_url),
        Some(item.raw_image_url),
        FileHostPublicity::Public,
        &**file_host,
    )
    .await
    .wrap_api_err("deleting old images")?;

    clear_project_cache_and_queue_search(
        &redis,
        &search_state,
        project_item.inner.id,
        project_item.inner.slug,
        None,
    )
    .await
    .wrap_api_err("executing `clear_project_cache_and_queue_search`")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Delete a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = NO_CONTENT))
)]
#[delete("/{id}")]
pub async fn project_delete(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<(), ApiError> {
    project_delete_internal(req, info, pool, redis, session_queue, search_state)
        .await
}

pub async fn project_delete_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    search_state: web::Data<SearchState>,
) -> Result<(), ApiError> {
    let (_, user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_DELETE,
    )
    .await
    .wrap_auth_err("authenticating API request")?;
    let string = info.into_inner().0;

    // In two cases, we return `The specified project does not exist!`:
    // - the project really doesn't exist
    // - the project is hidden from the user
    //
    // We use an `ApiError::Auth` for this case instead of a `ApiError::Request`,
    // because our permissions tests assert that failing under the 2nd use                  case
    // gives a 401 or 404, but `Request` gives only a 400.

    let project = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_internal_err("failed to get project")?
        .wrap_auth_err("the specified project does not exist")?;

    if !user.role.is_admin() {
        let (team_member, organization_team_member) =
            db_models::DBTeamMember::get_for_project_permissions(
                &project.inner,
                user.id.into(),
                &**pool,
            )
            .await
            .wrap_internal_err("failed to get user team member permissions")?;

        // Hide the project
        if team_member.is_none() && organization_team_member.is_none() {
            return Err(ApiError::Auth(eyre!(
                "The specified project does not exist!"
            )));
        }

        let permissions = ProjectPermissions::get_permissions_by_role(
            &user.role,
            &team_member,
            &organization_team_member,
        )
        .unwrap_or_default();

        if !permissions.contains(ProjectPermissions::DELETE_PROJECT) {
            return Err(ApiError::Auth(eyre::eyre!(
                "You don't have permission to delete this project!",
            )));
        }
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("failed to start transaction")?;
    delphi::tech_review_queue::remove_projects(
        &[project.inner.id],
        delphi::tech_review_queue::TechReviewRemovalReason::FileDeleted,
        &mut transaction,
    )
    .await
    .wrap_internal_err("removing project from technical review queue")?;

    // rejected & withheld projects are transferred to ghost so moderation data is preserved
    if matches!(
        project.inner.status,
        ProjectStatus::Rejected | ProjectStatus::Withheld
    ) {
        let deleted_user: db_ids::DBUserId = DELETED_USER.into();

        let deleted_slug = if let Some(slug) = &project.inner.slug {
            let candidate = format!(
                "{slug}--deleted-{}",
                ProjectId::from(project.inner.id)
            );
            if candidate.len() <= 255 {
                let taken = sqlx::query!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM mods
                        WHERE
                            (slug = LOWER($1) OR text_id_lower = LOWER($1))
                            AND id != $2
                    ) AS "exists!"
                    "#,
                    candidate,
                    project.inner.id as db_ids::DBProjectId,
                )
                .fetch_one(&mut transaction)
                .await
                .wrap_internal_err("checking deleted slug availability")?
                .exists;

                if !taken {
                    Some(candidate.to_lowercase())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        sqlx::query!(
            "
            UPDATE mods
            SET organization_id = NULL, slug = COALESCE($1, slug)
            WHERE id = $2
            ",
            deleted_slug,
            project.inner.id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err(
            "failed to detach project from organization and update slug",
        )?;

        let affected_user_ids = sqlx::query!(
            "
            DELETE FROM team_members
            WHERE team_id = $1
            RETURNING user_id
            ",
            project.inner.team_id as db_ids::DBTeamId,
        )
        .fetch(&mut transaction)
        .map_ok(|x| db_ids::DBUserId(x.user_id))
        .try_collect::<Vec<_>>()
        .await
        .wrap_internal_err("failed to remove project team members")?;

        let new_member_id = db_ids::generate_team_member_id(&mut transaction)
            .await
            .wrap_internal_err("failed to generate team member ID")?;
        DBTeamMember {
            id: new_member_id,
            team_id: project.inner.team_id,
            user_id: deleted_user,
            role: DEFAULT_ROLE.to_owned(),
            is_owner: true,
            permissions: ProjectPermissions::all(),
            organization_permissions: None,
            accepted: true,
            payouts_split: Decimal::ZERO,
            ordering: 0,
        }
        .insert(&mut transaction)
        .await
        .wrap_internal_err("failed to transfer project ownership to ghost")?;

        ThreadMessageBuilder {
            author_id: Some(deleted_user),
            body: MessageBody::Text {
                body: format!(
                    "Project transferred to Ghost when user account `{}` (`{}`) deleted this project",
                    user.username,
                    user.id
                ),
                private: true,
                replying_to: None,
                associated_images: Vec::new(),
            },
            thread_id: project.thread_id,
            hide_identity: false,
        }
        .insert(&mut transaction)
        .await
        .wrap_internal_err(
            "failed to insert project transfer thread message",
        )?;

        sqlx::query!(
            "
            DELETE FROM collections_mods
            WHERE mod_id = $1
            ",
            project.inner.id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("failed to delete project from collections_mods")?;

        sqlx::query!(
            "
            DELETE FROM mod_follows
            WHERE mod_id = $1
            ",
            project.inner.id as db_ids::DBProjectId,
        )
        .execute(&mut transaction)
        .await
        .wrap_internal_err("failed to delete project followers")?;

        transaction
            .commit()
            .await
            .wrap_internal_err("failed to commit transaction")?;

        let mut cache_user_ids = affected_user_ids;
        cache_user_ids.push(deleted_user);
        db_models::DBUser::clear_project_cache(&cache_user_ids, &redis)
            .await
            .wrap_internal_err("failed to clear user project cache")?;
        DBTeamMember::clear_cache(project.inner.team_id, &redis)
            .await
            .wrap_internal_err("clearing cached data from Redis")?;
        db_models::DBProject::clear_cache(
            project.inner.id,
            project.inner.slug.clone(),
            None,
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
        search_state
            .queue
            .push_project_removal(project.inner.id.into())
            .await;

        return Ok(());
    }

    let context = ImageContext::Project {
        project_id: Some(project.inner.id.into()),
    };
    let uploaded_images =
        db_models::DBImage::get_many_contexted(context, &mut transaction)
            .await
            .wrap_internal_err("failed to get project images")?;
    for image in uploaded_images {
        image_item::DBImage::remove(image.id, &mut transaction, &redis)
            .await
            .wrap_internal_err_with(|| {
                eyre!("failed to remove project image `{:?}`", image.id)
            })?;
    }

    sqlx::query!(
        "
        DELETE FROM collections_mods
        WHERE mod_id = $1
        ",
        project.inner.id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("failed to delete project from collections_mods")?;

    let result = db_models::DBProject::remove(
        project.inner.id,
        &mut transaction,
        &redis,
    )
    .await
    .wrap_internal_err("failed to remove project")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("failed to commit transaction")?;

    if result.is_some() {
        db_models::DBProject::clear_cache(
            project.inner.id,
            project.inner.slug,
            None,
            &redis,
        )
        .await
        .wrap_internal_err("clearing cached data from Redis")?;
        search_state
            .queue
            .push_project_removal(project.inner.id.into())
            .await;
        Ok(())
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}

/// Follow a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = NO_CONTENT))
)]
#[post("/{id}/follow")]
pub async fn project_follow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    project_follow_internal(req, info, pool, redis, session_queue).await
}

pub async fn project_follow_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let string = info.into_inner().0;

    let project = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    let user_id: db_ids::DBUserId = user.id.into();
    let project_id: db_ids::DBProjectId = project.inner.id;

    if !is_visible_project(&project.inner, &Some(user), &pool, false)
        .await
        .wrap_api_err("checking project visibility")?
    {
        return Err(ApiError::NotFound(eyre::eyre!("resource not found")));
    }

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let insert_result = sqlx::query!(
        "
            INSERT INTO mod_follows (follower_id, mod_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            ",
        user_id as db_ids::DBUserId,
        project_id as db_ids::DBProjectId
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("querying database for `project_follow_internal`")?;

    if insert_result.rows_affected() == 0 {
        return Err(ApiError::Request(eyre::eyre!(
            "you are already following this project",
        )));
    }

    sqlx::query!(
        "
            UPDATE mods
            SET follows = follows + 1
            WHERE id = $1
            ",
        project_id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("querying database for `project_follow_internal`")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    db_models::DBProject::clear_cache(
        project_id,
        project.inner.slug,
        None,
        &redis,
    )
    .await
    .wrap_internal_err("clearing cached project data")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Unfollow a project.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = NO_CONTENT))
)]
#[delete("/{id}/follow")]
pub async fn project_unfollow(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    project_unfollow_internal(req, info, pool, redis, session_queue).await
}

pub async fn project_unfollow_internal(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let string = info.into_inner().0;

    let project = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    let user_id: db_ids::DBUserId = user.id.into();
    let project_id = project.inner.id;

    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting database transaction")?;

    let delete_result = sqlx::query!(
        "
            DELETE FROM mod_follows
            WHERE follower_id = $1 AND mod_id = $2
            ",
        user_id as db_ids::DBUserId,
        project_id as db_ids::DBProjectId
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("querying database for `project_unfollow_internal`")?;

    if delete_result.rows_affected() == 0 {
        return Err(ApiError::Request(eyre::eyre!(
            "you are not following this project",
        )));
    }

    sqlx::query!(
        "
            UPDATE mods
            SET follows = GREATEST(follows - 1, 0)
            WHERE id = $1
            ",
        project_id as db_ids::DBProjectId,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("querying database for `project_unfollow_internal`")?;

    transaction
        .commit()
        .await
        .wrap_internal_err("committing database transaction")?;

    db_models::DBProject::clear_cache(
        project_id,
        project.inner.slug,
        None,
        &redis,
    )
    .await
    .wrap_internal_err("clearing cached project data")?;

    Ok(HttpResponse::NoContent().body(""))
}

/// Get a project's organization.
#[utoipa::path(
	context_path = "/project",
	tag = "projects", responses((status = OK, body = models::organizations::Organization))
)]
#[get("/{id}/organization")]
pub async fn project_get_organization(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<HttpResponse, ApiError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ | Scopes::ORGANIZATION_READ,
    )
    .await
    .map(|x| x.1)
    .ok();
    let user_id = current_user.as_ref().map(|x| x.id.into());

    let string = info.into_inner().0;
    let result = db_models::DBProject::get(&string, &**pool, &redis)
        .await
        .wrap_api_err("fetching project from database")?
        .wrap_request_err_with(|| {
            "the specified project does not exist!".to_string()
        })?;

    if !is_visible_project(&result.inner, &current_user, &pool, false)
        .await
        .wrap_api_err("checking project visibility")?
    {
        Err(ApiError::Request(eyre::eyre!(
            "The specified project does not exist!",
        )))
    } else if let Some(organization_id) = result.inner.organization_id {
        let organization =
            db_models::DBOrganization::get_id(organization_id, &**pool, &redis)
                .await
                .wrap_internal_err("fetching organization from database")?
                .wrap_request_err_with(|| {
                    "the attached organization does not exist!".to_string()
                })?;

        let members_data = DBTeamMember::get_from_team_full(
            organization.team_id,
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching team members from database")?;

        let users = crate::database::models::DBUser::get_many_ids(
            &members_data.iter().map(|x| x.user_id).collect::<Vec<_>>(),
            &**pool,
            &redis,
        )
        .await
        .wrap_internal_err("fetching users from database")?;
        let logged_in = current_user
            .as_ref()
            .and_then(|user| {
                members_data
                    .iter()
                    .find(|x| x.user_id == user.id.into() && x.accepted)
            })
            .is_some();
        let team_members: Vec<_> = members_data
            .into_iter()
            .filter(|x| {
                logged_in
                    || x.accepted
                    || user_id.is_some_and(
                        |y: crate::database::models::DBUserId| y == x.user_id,
                    )
            })
            .filter_map(|data| {
                users.iter().find(|x| x.id == data.user_id).map(|user| {
                    crate::models::teams::TeamMember::from(
                        data,
                        user.clone(),
                        !logged_in,
                    )
                })
            })
            .collect();

        let organization = models::organizations::Organization::from(
            organization,
            team_members,
        );
        Ok(HttpResponse::Ok().json(organization))
    } else {
        Err(ApiError::NotFound(eyre::eyre!("resource not found")))
    }
}
