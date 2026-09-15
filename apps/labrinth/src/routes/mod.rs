use crate::env::ENV;
use crate::util::cors::default_cors;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use futures::FutureExt;
use utoipa::openapi::extensions::ExtensionsBuilder;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

pub mod debug;
pub mod internal;
pub mod v2;
pub mod v2_reroute;
pub mod v3;

pub mod analytics;
mod index;
mod maven;
mod not_found;
mod updates;

pub use self::not_found::not_found;

// utoipa-specific struct to use a value_type for docs.
/// A sha1 or sha512 hash.
pub struct FileHash;

impl utoipa::PartialSchema for FileHash {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .min_length(Some(40))
            .max_length(Some(128))
            .pattern(Some("^([A-Fa-f0-9]{40}|[A-Fa-f0-9]{128})$"))
            .examples([serde_json::json!(
                "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"
            )])
            .description(Some("A sha1 or sha512 hash."))
            .build()
            .into()
    }
}

impl utoipa::ToSchema for FileHash {}

// utoipa-specific struct to use a value_type for docs.
/// A hashing algorithm (sha1 or sha256)
pub struct HashAlgorithm;

impl utoipa::PartialSchema for HashAlgorithm {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .enum_values(Some([
                serde_json::json!("sha1"),
                serde_json::json!("sha512"),
            ]))
            .examples([serde_json::json!("sha1")])
            .description(Some("A supported hashing algorithm."))
            .build()
            .into()
    }
}

impl utoipa::ToSchema for HashAlgorithm {}

pub(crate) fn prefix_openapi_paths(
    openapi: &mut utoipa::openapi::OpenApi,
    prefix: &str,
    should_skip: impl Fn(&str) -> bool,
) {
    let paths = std::mem::take(&mut openapi.paths.paths);
    openapi.paths.paths = paths
        .into_iter()
        .map(|(path, item)| {
            if should_skip(&path) || path.starts_with(prefix) {
                (path, item)
            } else {
                (format!("{prefix}{}", normalize_openapi_path(&path)), item)
            }
        })
        .collect();
}

fn normalize_openapi_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

pub(crate) struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        let mut bearer_auth = HttpBuilder::new()
			.scheme(HttpAuthScheme::Bearer)
			.description(Some(
				"Use a personal access token. Example: `mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic`.",
			))
			.build();
        bearer_auth.extensions = Some(
			ExtensionsBuilder::new()
				.add(
					"x-example",
					"mrp_RNtLRSPmGj2pd1v1ubi52nX7TJJM9sznrmwhAuj511oe4t1jAqAQ3D6Wc8Ic",
				)
				.build(),
		);

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(bearer_auth),
        );
    }
}

pub fn root_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .wrap(default_cors())
            .wrap_fn(|req, _srv| {
            async {
                Ok(req.into_response(
                    HttpResponse::Gone()
                        .content_type("application/json")
                        .body(r#"{"error":"api_deprecated","description":"You are using an application that uses an outdated version of Modrinth's API. Please either update it or switch to another application. For developers: https://docs.modrinth.com/api/#versioning"}"#)
                ))
            }.boxed_local()
        })
    );
    cfg.service(
        web::scope("")
            .wrap(default_cors())
            .service(index::index_get)
            .service(index::build_get)
            .service(Files::new("/", "assets/")),
    );
}

pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/maven")
            .wrap(default_cors())
            .configure(maven::config),
    );
    cfg.service(
        web::scope("/updates")
            .wrap(default_cors())
            .configure(updates::config),
    );
    cfg.service(
        web::scope("/analytics")
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| {
                        let allowed_origins = &ENV.ANALYTICS_ALLOWED_ORIGINS;
                        allowed_origins.contains(&"*".to_string())
                            || allowed_origins.contains(
                                &origin
                                    .to_str()
                                    .unwrap_or_default()
                                    .to_string(),
                            )
                    })
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::ACCEPT,
                        actix_web::http::header::CONTENT_TYPE,
                    ])
                    .max_age(3600),
            )
            .configure(analytics::config),
    );
}

/// Error when calling an HTTP endpoint.
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    /// Error occurred on the server side, which the caller has no fault in.
    #[error(transparent)]
    Internal(eyre::Report),
    /// Caller made an invalid or malformed request.
    #[error(transparent)]
    Request(eyre::Report),
    /// Caller attempted a request which they are not allowed to make.
    #[error(transparent)]
    Auth(eyre::Report),
    /// The requested resource does not exist.
    #[error(transparent)]
    NotFound(eyre::Report),
    /// The request conflicts with the current state of the resource.
    #[error(transparent)]
    Conflict(eyre::Report),
    /// A service dependency failed to complete the request.
    #[error(transparent)]
    FailedDependency(eyre::Report),
    /// The request requires a precondition.
    #[error(transparent)]
    PreconditionRequired(eyre::Report),
    /// A request precondition was not met.
    #[error(transparent)]
    PreconditionFailed(eyre::Report),
    /// The caller exceeded a request rate limit.
    #[error(transparent)]
    RateLimit(eyre::Report),
}

impl ApiError {
    /// Adds context to the contained report while preserving the HTTP status.
    pub(crate) fn wrap_err<D>(self, msg: D) -> Self
    where
        D: Send + Sync + std::fmt::Debug + std::fmt::Display + 'static,
    {
        match self {
            Self::Internal(report) => Self::Internal(report.wrap_err(msg)),
            Self::Request(report) => Self::Request(report.wrap_err(msg)),
            Self::Auth(report) => Self::Auth(report.wrap_err(msg)),
            Self::NotFound(report) => Self::NotFound(report.wrap_err(msg)),
            Self::Conflict(report) => Self::Conflict(report.wrap_err(msg)),
            Self::FailedDependency(report) => {
                Self::FailedDependency(report.wrap_err(msg))
            }
            Self::PreconditionRequired(report) => {
                Self::PreconditionRequired(report.wrap_err(msg))
            }
            Self::PreconditionFailed(report) => {
                Self::PreconditionFailed(report.wrap_err(msg))
            }
            Self::RateLimit(report) => Self::RateLimit(report.wrap_err(msg)),
        }
    }

    pub fn as_api_error<'a>(&self) -> crate::models::error::ApiError<'a> {
        let report = match self {
            Self::Internal(report)
            | Self::Request(report)
            | Self::Auth(report)
            | Self::NotFound(report)
            | Self::Conflict(report)
            | Self::FailedDependency(report)
            | Self::PreconditionRequired(report)
            | Self::PreconditionFailed(report)
            | Self::RateLimit(report) => report,
        };
        let details = report
            .chain()
            .skip(1)
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        let validation = report
            .downcast_ref::<v3::projects::validate::ProjectValidationError>();

        crate::models::error::ApiError {
            error: match self {
                Self::Internal(..) => "internal_error",
                Self::Request(..) => "request_error",
                Self::Auth(..) => "auth_error",
                Self::NotFound(..) => "not_found",
                Self::Conflict(..) => "conflict",
                Self::FailedDependency(..) => "failed_dependency",
                Self::PreconditionRequired(..) => "precondition_required",
                Self::PreconditionFailed(..) => "precondition_failed",
                Self::RateLimit(..) => "ratelimit_error",
            },
            description: report.to_string(),
            details: validation
                .map(|error| serde_json::json!({ "nags": error.0 }))
                .or_else(|| {
                    (!details.is_empty()).then(|| serde_json::json!(details))
                }),
        }
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(..) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Request(..) => StatusCode::BAD_REQUEST,
            Self::Auth(..) => StatusCode::UNAUTHORIZED,
            Self::NotFound(..) => StatusCode::NOT_FOUND,
            Self::Conflict(..) => StatusCode::CONFLICT,
            Self::FailedDependency(..) => StatusCode::FAILED_DEPENDENCY,
            Self::PreconditionRequired(..) => StatusCode::PRECONDITION_REQUIRED,
            Self::PreconditionFailed(..) => StatusCode::PRECONDITION_FAILED,
            Self::RateLimit(..) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.as_api_error())
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;

    #[test]
    fn api_error_serializes_source_chain_as_details() {
        let error = ApiError::Request(
            eyre::eyre!("root cause")
                .wrap_err("intermediate context")
                .wrap_err("request failed"),
        );

        let response = error.as_api_error();

        assert_eq!(response.description, "request failed");
        assert_eq!(
            response.details,
            Some(serde_json::json!(["intermediate context", "root cause"])),
        );
    }
}
