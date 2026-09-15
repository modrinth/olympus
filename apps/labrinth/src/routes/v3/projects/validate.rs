use actix_web::{HttpRequest, get, web};
use eyre::eyre;
use serde::Serialize;
use xredis::RedisPool;

use crate::auth::get_user_from_headers;
use crate::database::models::DBProjectId;
use crate::database::models::project_item::ProjectQueryResult;
use crate::database::{
    PgPool, PgTransaction, ReadOnlyPgPool, models as db_models,
};
use crate::models::ids::ProjectId;
use crate::models::pats::Scopes;
use crate::models::projects::{Project, Version};
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context as _;
use crate::validate::project::{
    ProjectNag, has_required_nags_with_context,
    validate_with_context as validate_project,
};

#[derive(Serialize, utoipa::ToSchema)]
pub struct ProjectValidationResponse {
    pub nags: Vec<ProjectNag>,
}

pub(crate) async fn ensure_project_is_valid_for_review(
    project_id: DBProjectId,
    pool: &PgPool,
    transaction: &mut PgTransaction<'_>,
    redis: &RedisPool,
) -> Result<ProjectQueryResult, ApiError> {
    let mut projects = db_models::DBProject::get_many_uncached(
        &[ProjectId::from(project_id)],
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project for review validation")?;
    let reloaded_project =
        projects.pop().wrap_not_found_err("resource not found")?;
    let versions = db_models::DBVersion::get_many_uncached(
        &reloaded_project.versions,
        &mut *transaction,
        redis,
    )
    .await
    .wrap_internal_err("reloading project versions for review validation")?
    .into_iter()
    .map(Version::from)
    .collect::<Vec<_>>();
    let available_categories =
        db_models::categories::Category::list(&**pool, redis)
            .await
            .wrap_internal_err("fetching project categories")?;
    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        reloaded_project.inner.id,
        false,
        &mut *transaction,
    )
    .await
    .wrap_internal_err("fetching project disclosures")?
    .into_iter()
    .map(|disclosure| disclosure.disclosure)
    .collect::<Vec<_>>();
    let project = Project::from(reloaded_project.clone());

    let has_required_nags = web::block(move || {
        has_required_nags_with_context(
            &project,
            &versions,
            &available_categories,
            &disclosures,
        )
    })
    .await
    .wrap_internal_err("validating project for review")?;
    if has_required_nags {
        return Err(ApiError::Request(eyre!(
            "project must have no required validation nags before or while under review"
        )));
    }

    Ok(reloaded_project)
}

/// Validate that a project is ready to be submitted for review.
#[utoipa::path(
	context_path = "/project",
	tag = "projects",
	responses((status = OK, body = ProjectValidationResponse))
)]
#[get("/{id}/validate")]
pub async fn validate(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<web::Json<ProjectValidationResponse>, ApiError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;

    let project =
        db_models::DBProject::get(&info.into_inner().0, &***ro_pool, &redis)
            .await
            .wrap_internal_err("fetching project from database")?
            .wrap_not_found_err("resource not found")?;

    let (team_member, organization_team_member) =
        db_models::DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.id.into(),
            &***ro_pool,
        )
        .await
        .wrap_internal_err("fetching project permissions")?;

    if ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .is_none()
    {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to validate this project"
        )));
    }

    let versions =
        db_models::DBVersion::get_many(&project.versions, &***ro_pool, &redis)
            .await
            .wrap_internal_err("fetching project versions from database")?
            .into_iter()
            .map(Version::from)
            .collect::<Vec<_>>();
    let available_categories =
        db_models::categories::Category::list(&**pool, &redis)
            .await
            .wrap_internal_err("fetching project categories")?;
    let disclosures = db_models::DBProjectDisclosure::get_many_for_project(
        project.inner.id,
        false,
        &***ro_pool,
    )
    .await
    .wrap_internal_err("fetching project disclosures")?
    .into_iter()
    .map(|disclosure| disclosure.disclosure)
    .collect::<Vec<_>>();
    let project = Project::from(project);

    let nags = web::block(move || {
        validate_project(
            &project,
            &versions,
            &available_categories,
            &disclosures,
        )
    })
    .await
    .wrap_internal_err("validating project")?;
    Ok(web::Json(ProjectValidationResponse { nags }))
}
