use actix_http::StatusCode;
use actix_web::test;
use common::api_v3::ApiV3;
use common::api_v3::request_data::get_public_project_creation_data;
use common::database::*;
use common::dummy_data::DUMMY_CATEGORIES;

use crate::common::api_common::models::CommonProject;
use crate::common::api_common::request_data::ProjectCreationRequestData;
use crate::common::api_common::{
    Api, ApiProject, ApiTags, ApiTeams, ApiVersion, AppendsOptionalPat,
};
use crate::common::dummy_data::{
    DummyImage, DummyOrganizationZeta, DummyProjectAlpha, DummyProjectBeta,
    TestFile,
};
use ariadne::ids::base62_impl::parse_base62;
use common::environment::{
    TestEnvironment, with_test_environment, with_test_environment_all,
};
use common::permissions::{PermissionsTest, PermissionsTestContext};
use futures::StreamExt;
use hex::ToHex;
use labrinth::database::models::DBProjectId;
use labrinth::database::models::project_item::{
    PROJECTS_NAMESPACE, PROJECTS_SLUGS_NAMESPACE, ProjectQueryResult,
};
use labrinth::models::ids::ProjectId;
use labrinth::models::projects::ProjectStatus;
use labrinth::models::teams::ProjectPermissions;
use labrinth::util::actix::{MultipartSegment, MultipartSegmentData};
use serde_json::json;
use sha1::Digest;
use xredis::RedisValue;

pub mod common;

#[actix_rt::test]
async fn test_get_project() {
    // Test setup and dummy data
    with_test_environment_all(None, |test_env| async move {
        let DummyProjectAlpha {
            project_id: alpha_project_id,
            project_slug: alpha_project_slug,
            version_id: alpha_version_id,
            ..
        } = &test_env.dummy.project_alpha;
        let DummyProjectBeta {
            project_id: beta_project_id,
            ..
        } = &test_env.dummy.project_beta;

        let api = &test_env.api;

        // Perform request on dummy data
        let resp = api.get_project(alpha_project_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;

        assert_eq!(body["id"], json!(alpha_project_id));
        assert_eq!(body["slug"], json!(alpha_project_slug));
        let versions = body["versions"].as_array().unwrap();
        assert_eq!(versions[0], json!(alpha_version_id));

        // Confirm that the request was cached
        let mut redis_pool = test_env.db.redis_pool.connect().await.unwrap();
        let slug_key = redis_pool
            .key()
            .entity(PROJECTS_SLUGS_NAMESPACE, alpha_project_slug);
        assert_eq!(
            redis_pool
                .get(&slug_key)
                .await
                .unwrap()
                .and_then(|x| x.parse::<i64>().ok()),
            Some(parse_base62(alpha_project_id).unwrap() as i64)
        );

        let project_key = redis_pool.key().entity(
            PROJECTS_NAMESPACE,
            parse_base62(alpha_project_id).unwrap(),
        );
        let cached_project: RedisValue<
            ProjectQueryResult,
            DBProjectId,
            String,
        > = redis_pool
            .get_deserialized(&project_key)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            cached_project.value().inner.slug.as_ref(),
            Some(alpha_project_slug)
        );

        // Make the request again, this time it should be cached
        let resp = api.get_project(alpha_project_id, USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], json!(alpha_project_id));
        assert_eq!(body["slug"], json!(alpha_project_slug));

        // Request should fail on non-existent project
        let resp = api.get_project("nonexistent", USER_USER_PAT).await;
        assert_status!(&resp, StatusCode::NOT_FOUND);

        // Similarly, request should fail on non-authorized user, on a yet-to-be-approved or hidden project, with a 404 (hiding the existence of the project)
        let resp = api.get_project(beta_project_id, ENEMY_USER_PAT).await;
        assert_status!(&resp, StatusCode::NOT_FOUND);
    })
    .await;
}

#[actix_rt::test]
async fn test_add_remove_project() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            // Generate test project data.
            let mut json_data = api
                .get_public_project_creation_data_json(
                    "demo",
                    Some(&TestFile::BasicMod),
                )
                .await;

            // Basic json
            let json_segment = MultipartSegment {
                name: "data".to_string(),
                filename: None,
                content_type: Some("application/json".to_string()),
                data: MultipartSegmentData::Text(
                    serde_json::to_string(&json_data).unwrap(),
                ),
            };

            // Basic json, with a different file
            json_data["initial_versions"][0]["file_parts"][0] =
                json!("basic-mod-different.jar");
            let json_diff_file_segment = MultipartSegment {
                data: MultipartSegmentData::Text(
                    serde_json::to_string(&json_data).unwrap(),
                ),
                ..json_segment.clone()
            };

            // Basic json, with a different file, and a different slug
            json_data["slug"] = json!("new_demo");
            json_data["initial_versions"][0]["file_parts"][0] =
                json!("basic-mod-different.jar");
            let json_diff_slug_file_segment = MultipartSegment {
                data: MultipartSegmentData::Text(
                    serde_json::to_string(&json_data).unwrap(),
                ),
                ..json_segment.clone()
            };

            let basic_mod_file = TestFile::BasicMod;
            let basic_mod_different_file = TestFile::BasicModDifferent;

            // Basic file
            let file_segment = MultipartSegment {
                // 'Basic'
                name: basic_mod_file.filename(),
                filename: Some(basic_mod_file.filename()),
                content_type: basic_mod_file.content_type(),
                data: MultipartSegmentData::Binary(basic_mod_file.bytes()),
            };

            // Differently named file, with the SAME content (for hash testing)
            let file_diff_name_segment = MultipartSegment {
                // 'Different'
                name: basic_mod_different_file.filename(),
                filename: Some(basic_mod_different_file.filename()),
                content_type: basic_mod_different_file.content_type(),
                // 'Basic'
                data: MultipartSegmentData::Binary(basic_mod_file.bytes()),
            };

            // Differently named file, with different content
            let file_diff_name_content_segment = MultipartSegment {
                // 'Different'
                name: basic_mod_different_file.filename(),
                filename: Some(basic_mod_different_file.filename()),
                content_type: basic_mod_different_file.content_type(),
                data: MultipartSegmentData::Binary(
                    basic_mod_different_file.bytes(),
                ),
            };

            // Add a project- simple, should work.
            let resp = api
                .create_project(
                    ProjectCreationRequestData {
                        slug: "demo".to_string(),
                        segment_data: vec![
                            json_segment.clone(),
                            file_segment.clone(),
                        ],
                        jar: None, // File not needed at this point
                    },
                    USER_USER_PAT,
                )
                .await;

            assert_status!(&resp, StatusCode::OK);

            // Get the project we just made, and confirm that it's correct
            let project = api
                .get_project_deserialized_common("demo", USER_USER_PAT)
                .await;
            assert!(project.versions.len() == 1);
            let uploaded_version_id = project.versions[0];

            // Checks files to ensure they were uploaded and correctly identify the file
            let hash = sha1::Sha1::digest(basic_mod_file.bytes())
                .encode_hex::<String>();
            let version = api
                .get_version_from_hash_deserialized_common(
                    &hash,
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(version.id, uploaded_version_id);

            // Reusing with a different slug and the same file should fail
            // Even if that file is named differently
            let resp = api
                .create_project(
                    ProjectCreationRequestData {
                        slug: "demo".to_string(),
                        segment_data: vec![
                            json_diff_slug_file_segment.clone(),
                            file_diff_name_segment.clone(),
                        ],
                        jar: None, // File not needed at this point
                    },
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Reusing with the same slug and a different file should fail
            let resp = api
                .create_project(
                    ProjectCreationRequestData {
                        slug: "demo".to_string(),
                        segment_data: vec![
                            json_diff_file_segment.clone(),
                            file_diff_name_content_segment.clone(),
                        ],
                        jar: None, // File not needed at this point
                    },
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Different slug, different file should succeed
            let resp = api
                .create_project(
                    ProjectCreationRequestData {
                        slug: "demo".to_string(),
                        segment_data: vec![
                            json_diff_slug_file_segment.clone(),
                            file_diff_name_content_segment.clone(),
                        ],
                        jar: None, // File not needed at this point
                    },
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Get
            let project = api
                .get_project_deserialized_common("demo", USER_USER_PAT)
                .await;
            let id = project.id.to_string();

            // Remove the project
            let resp = test_env.api.remove_project("demo", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm that the project is gone from the cache
            let mut redis_pool =
                test_env.db.redis_pool.connect().await.unwrap();
            let slug_key =
                redis_pool.key().entity(PROJECTS_SLUGS_NAMESPACE, "demo");
            let id_key = redis_pool.key().entity(PROJECTS_SLUGS_NAMESPACE, &id);
            assert_eq!(
                redis_pool
                    .get(&slug_key)
                    .await
                    .unwrap()
                    .and_then(|x| x.parse::<i64>().ok()),
                None
            );
            assert_eq!(
                redis_pool
                    .get(&id_key)
                    .await
                    .unwrap()
                    .and_then(|x| x.parse::<i64>().ok()),
                None
            );

            // Old slug no longer works
            let resp = api.get_project("demo", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NOT_FOUND);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_patch_project() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let alpha_project_slug = &test_env.dummy.project_alpha.project_slug;
            let beta_project_slug = &test_env.dummy.project_beta.project_slug;

            // First, we do some patch requests that should fail.
            // Failure because the user is not authorized.
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "name": "Test_Add_Project project - test 1",
                    }),
                    ENEMY_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // Failure because we are setting URL fields to invalid urls.
            for url_type in ["issues", "source", "wiki", "discord"] {
                let resp = api
                    .edit_project(
                        alpha_project_slug,
                        json!({
                            "link_urls": {
                                url_type: "not a url",
                            },
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "summary": "",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);
            let body: serde_json::Value = test::read_body_json(resp).await;
            assert_eq!(
                body["description"],
                "field summary failed validation with error: length"
            );
            assert!(body.get("details").is_none());

            // Failure because these are illegal requested statuses for a normal user.
            for req in ["unknown", "processing", "withheld", "scheduled"] {
                let resp = api
                    .edit_project(
                        alpha_project_slug,
                        json!({
                            "requested_status": req,
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Failure because these should not be able to be set by a non-mod
            for key in ["moderation_message", "moderation_message_body"] {
                let resp = api
                    .edit_project(
                        alpha_project_slug,
                        json!({
                            key: "test",
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::UNAUTHORIZED);

                // (should work for a mod, though)
                let resp = api
                    .edit_project(
                        alpha_project_slug,
                        json!({
                            key: "test",
                        }),
                        MOD_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::NO_CONTENT);
            }

            // Failed patch to alpha slug:
            // - slug collision with beta
            // - too short slug
            // - too long slug
            // - not url safe slug
            // - not url safe slug
            for slug in [
                beta_project_slug,
                "a",
                &"a".repeat(100),
                "not url safe%&^!#$##!@#$%^&*()",
            ] {
                let resp = api
                    .edit_project(
                        alpha_project_slug,
                        json!({
                            "slug": slug, // the other dummy project has this slug
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::BAD_REQUEST);
            }

            // Not allowed to directly set status, as 'beta_project_slug' (the other project) is "processing" and cannot have its status changed like this.
            let resp = api
                .edit_project(
                    beta_project_slug,
                    json!({
                        "status": "private"
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::UNAUTHORIZED);

            // Successful request to patch many fields.
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "slug": "newslug",
                        "categories": [DUMMY_CATEGORIES[0]],
                        "license_id": "MIT",
                        "link_urls":
                            {
                                "patreon": "https://patreon.com",
                                "issues": "https://github.com",
                                "discord": "https://discord.gg",
                                "wiki": "https://wiki.com"
                            }
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Old slug no longer works
            let resp = api.get_project(alpha_project_slug, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NOT_FOUND);

            // New slug does work
            let project =
                api.get_project_deserialized("newslug", USER_USER_PAT).await;

            assert_eq!(project.slug.unwrap(), "newslug");
            assert_eq!(project.categories, vec![DUMMY_CATEGORIES[0]]);
            assert_eq!(project.license.id, "MIT");

            let link_urls = project.link_urls;
            assert_eq!(link_urls.len(), 4);
            assert_eq!(link_urls["patreon"].platform, "patreon");
            assert_eq!(link_urls["patreon"].url, "https://patreon.com");
            assert!(link_urls["patreon"].donation);
            assert_eq!(link_urls["issues"].platform, "issues");
            assert_eq!(link_urls["issues"].url, "https://github.com");
            assert!(!link_urls["issues"].donation);
            assert_eq!(link_urls["discord"].platform, "discord");
            assert_eq!(link_urls["discord"].url, "https://discord.gg");
            assert!(!link_urls["discord"].donation);
            assert_eq!(link_urls["wiki"].platform, "wiki");
            assert_eq!(link_urls["wiki"].url, "https://wiki.com");
            assert!(!link_urls["wiki"].donation);

            // Unset the set link_urls
            let resp = api
                .edit_project(
                    "newslug",
                    json!({
                        "link_urls":
                            {
                                "issues": null,
                            }
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);
            let project =
                api.get_project_deserialized("newslug", USER_USER_PAT).await;
            assert_eq!(project.link_urls.len(), 3);
            assert!(!project.link_urls.contains_key("issues"));
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_submit_invalid_project_for_review() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "draft" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let project_before = api
                .get_project_deserialized(project_slug, USER_USER_PAT)
                .await;
            assert_eq!(project_before.status, ProjectStatus::Draft);
            let original_description = project_before.description;

            let response = api
                .edit_project(
                    project_slug,
                    json!({
                        "description": "",
                        "status": "processing",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let project_after = api
                .get_project_deserialized(project_slug, USER_USER_PAT)
                .await;
            assert_eq!(project_after.status, ProjectStatus::Draft);
            assert_eq!(project_after.description, original_description);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_edit_invalid_project_in_review_rolls_back() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "processing" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let project_before = api
                .get_project_deserialized(project_slug, USER_USER_PAT)
                .await;
            assert_eq!(project_before.status, ProjectStatus::Processing);

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "description": "" }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let project_after = api
                .get_project_deserialized(project_slug, USER_USER_PAT)
                .await;
            assert_eq!(project_after.status, ProjectStatus::Processing);
            assert_eq!(project_after.description, project_before.description);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_leaving_review_skips_validation() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;
            let original_project = api
                .get_project_deserialized(project_slug, USER_USER_PAT)
                .await;

            for (status, expected_status) in [
                ("draft", ProjectStatus::Draft),
                ("rejected", ProjectStatus::Rejected),
            ] {
                let response = api
                    .edit_project(
                        project_slug,
                        json!({
                            "description": original_project.description.clone(),
                            "status": "processing",
                        }),
                        ADMIN_USER_PAT,
                    )
                    .await;
                assert_status!(&response, StatusCode::NO_CONTENT);

                let response = api
                    .edit_project(
                        project_slug,
                        json!({
                            "description": "",
                            "status": status,
                        }),
                        ADMIN_USER_PAT,
                    )
                    .await;
                assert_status!(&response, StatusCode::NO_CONTENT);

                let project = api
                    .get_project_deserialized(project_slug, USER_USER_PAT)
                    .await;
                assert_eq!(project.status, expected_status);
                assert!(project.description.is_empty());
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_description_similarity_to_summary() {
    with_test_environment(
		None,
		|test_env: TestEnvironment<ApiV3>| async move {
			let api = &test_env.api;
			let project_slug = &test_env.dummy.project_alpha.project_slug;
			let summary = "Explore new worlds with configurable tools and adventures.";
			let response = api
				.edit_project(
					project_slug,
					json!({ "status": "draft", "summary": summary }),
					ADMIN_USER_PAT,
				)
				.await;
			assert_status!(&response, StatusCode::NO_CONTENT);

			for (description, expected_match) in [
				(summary.to_string(), true),
				(format!("**{}**\n\n```yaml\nsetting: true\n```", summary.to_uppercase()), true),
				(format!("{summary} Players can discover custom structures, configure individual features, and follow detailed installation instructions for their preferred loader."), false),
				(String::new(), false),
			] {
				let response = api
					.edit_project(
						project_slug,
						json!({ "description": description }),
						USER_USER_PAT,
					)
					.await;
				assert_status!(&response, StatusCode::NO_CONTENT);

				let request = test::TestRequest::get()
					.uri(&format!("/v3/project/{project_slug}/validate"))
					.append_pat(USER_USER_PAT)
					.to_request();
				let response = api.call(request).await;
				assert_status!(&response, StatusCode::OK);
				let validation: serde_json::Value = test::read_body_json(response).await;
				let matching_nag = validation["nags"]
					.as_array()
					.unwrap()
					.iter()
					.find(|nag| nag["kind"] == "project_description_matches_summary");
				assert_eq!(matching_nag.is_some(), expected_match, "{description}");
				if let Some(nag) = matching_nag {
					assert_eq!(nag["severity"], "required");
				}
			}
		},
	)
	.await;
}

#[actix_rt::test]
async fn test_plugin_and_datapack_validation_use_mod_tags() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let mod_categories = api
                .get_categories_deserialized_common()
                .await
                .into_iter()
                .filter(|category| category.project_type == "mod")
                .map(|category| category.name)
                .collect::<Vec<_>>();
            assert!(mod_categories.len() > 3);

            for (project_type, loader, file) in [
                ("plugin", "bukkit", TestFile::build_random_plugin()),
                (
                    "datapack",
                    "datapack",
                    TestFile::build_random_datapack(),
                ),
            ] {
                let slug = format!("all-tags-selected-{project_type}");
                let modify_json = serde_json::from_value(json!([
                    { "op": "replace", "path": "/is_draft", "value": true },
                    { "op": "replace", "path": "/categories", "value": mod_categories[..3] },
                    { "op": "add", "path": "/additional_categories", "value": mod_categories[3..] },
                    { "op": "replace", "path": "/initial_versions/0/loaders", "value": [loader] },
                ]))
                .unwrap();
                let creation_data = get_public_project_creation_data(
                    &slug,
                    Some(file),
                    Some(modify_json),
                );
                let response =
                    api.create_project(creation_data, USER_USER_PAT).await;
                assert_status!(&response, StatusCode::OK);

                let request = test::TestRequest::get()
                    .uri(&format!("/v3/project/{slug}/validate"))
                    .append_pat(USER_USER_PAT)
                    .to_request();
                let response = api.call(request).await;
                assert_status!(&response, StatusCode::OK);
                let validation: serde_json::Value =
                    test::read_body_json(response).await;
                let all_tags_selected = validation["nags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|nag| nag["kind"] == "all_tags_selected")
                    .expect("all shared mod tags should be detected as selected");

                assert_eq!(
                    all_tags_selected["details"]["total_available_tags"],
                    mod_categories.len()
                );
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_zero_available_tags_are_not_all_selected() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let creation_data = get_public_project_creation_data(
                "project-without-an-inferred-type",
                None,
                None,
            );
            let response =
                api.create_project(creation_data, USER_USER_PAT).await;
            assert_status!(&response, StatusCode::OK);

            let request = test::TestRequest::get()
                .uri("/v3/project/project-without-an-inferred-type/validate")
                .append_pat(USER_USER_PAT)
                .to_request();
            let response = api.call(request).await;
            assert_status!(&response, StatusCode::OK);
            let validation: serde_json::Value =
                test::read_body_json(response).await;

            assert!(
                validation["nags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|nag| nag["kind"] != "all_tags_selected")
            );
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_add_invalid_gallery_item_in_review_rolls_back() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "processing" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let project_before =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_before: serde_json::Value =
                test::read_body_json(project_before).await;

            let response = api
                .add_gallery_item(
                    project_slug,
                    DummyImage::SmallIcon.get_icon_data(),
                    false,
                    None,
                    Some("%EF%BC%A1".to_string()),
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let project_after =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_after: serde_json::Value =
                test::read_body_json(project_after).await;
            assert_eq!(project_after["gallery"], project_before["gallery"]);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_edit_invalid_gallery_item_in_review_rolls_back() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;

            let response = api
                .add_gallery_item(
                    project_slug,
                    DummyImage::SmallIcon.get_icon_data(),
                    true,
                    None,
                    Some("valid-gallery-description".to_string()),
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "processing" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let project_before =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_before: serde_json::Value =
                test::read_body_json(project_before).await;
            let gallery_url =
                project_before["gallery"][0]["url"].as_str().unwrap();

            let response = api
                .edit_gallery_item(
                    project_slug,
                    gallery_url,
                    vec![("name".to_string(), "Ａ".to_string())]
                        .into_iter()
                        .collect(),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let project_after =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_after: serde_json::Value =
                test::read_body_json(project_after).await;
            assert_eq!(project_after["gallery"], project_before["gallery"]);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_delete_gallery_item_from_invalid_project_in_review_rolls_back() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;

            let response = api
                .add_gallery_item(
                    project_slug,
                    DummyImage::SmallIcon.get_icon_data(),
                    true,
                    None,
                    Some("valid-gallery-description".to_string()),
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let response = api
                .edit_project(
                    project_slug,
                    json!({
                        "description": "",
                        "status": "draft",
                    }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "processing" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let project_before =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_before: serde_json::Value =
                test::read_body_json(project_before).await;
            let gallery_url =
                project_before["gallery"][0]["url"].as_str().unwrap();

            let response = api
                .remove_gallery_item(project_slug, gallery_url, USER_USER_PAT)
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let project_after =
                api.get_project(project_slug, USER_USER_PAT).await;
            let project_after: serde_json::Value =
                test::read_body_json(project_after).await;
            assert_eq!(project_after["gallery"], project_before["gallery"]);
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_edit_invalid_disclosures_in_review_rolls_back() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let project_slug = &test_env.dummy.project_alpha.project_slug;
            let disclosures_uri =
                format!("/v3/project/{project_slug}/disclosures");

            let response = api
                .edit_project(
                    project_slug,
                    json!({ "status": "processing" }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::NO_CONTENT);

            let request = test::TestRequest::get()
                .uri(&disclosures_uri)
                .append_pat(USER_USER_PAT)
                .to_request();
            let response = api.call(request).await;
            assert_status!(&response, StatusCode::OK);
            let disclosures_before: serde_json::Value =
                test::read_body_json(response).await;

            let request = test::TestRequest::patch()
                .uri(&disclosures_uri)
                .append_pat(USER_USER_PAT)
                .set_json(json!({
                    "set": [{
                        "type": "advertisements",
                        "note": "<strong>sponsored</strong>",
                    }],
                    "remove": [],
                }))
                .to_request();
            let response = api.call(request).await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let request = test::TestRequest::get()
                .uri(&disclosures_uri)
                .append_pat(USER_USER_PAT)
                .to_request();
            let response = api.call(request).await;
            assert_status!(&response, StatusCode::OK);
            let disclosures_after: serde_json::Value =
                test::read_body_json(response).await;
            assert_eq!(disclosures_after, disclosures_before);
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_patch_v3() {
    // Hits V3-specific patchable fields
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let alpha_project_slug = &test_env.dummy.project_alpha.project_slug;

            // Successful request to patch many fields.
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "name": "New successful title",
                        "summary": "New successful summary",
                        "description": "New successful description",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let project = api
                .get_project_deserialized(alpha_project_slug, USER_USER_PAT)
                .await;

            assert_eq!(project.name, "New successful title");
            assert_eq!(project.summary, "New successful summary");
            assert_eq!(project.description, "New successful description");
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_categories() {
    with_test_environment_all(None, |test_env| async move {
        let api = &test_env.api;
        let alpha_project_id: &str = &test_env.dummy.project_alpha.project_id;
        let beta_project_id: &str = &test_env.dummy.project_beta.project_id;

        let resp = api
            .edit_project_bulk(
                &[alpha_project_id, beta_project_id],
                json!({
                    "categories": [DUMMY_CATEGORIES[0], DUMMY_CATEGORIES[3]],
                    "add_categories": [DUMMY_CATEGORIES[1], DUMMY_CATEGORIES[2]],
                    "remove_categories": [DUMMY_CATEGORIES[3]],
                    "additional_categories": [DUMMY_CATEGORIES[4], DUMMY_CATEGORIES[6]],
                    "add_additional_categories": [DUMMY_CATEGORIES[5]],
                    "remove_additional_categories": [DUMMY_CATEGORIES[6]],
                }),
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        let alpha_body = api
            .get_project_deserialized_common(alpha_project_id, ADMIN_USER_PAT)
            .await;
        assert_eq!(alpha_body.categories, DUMMY_CATEGORIES[0..=2]);
        assert_eq!(alpha_body.additional_categories, DUMMY_CATEGORIES[4..=5]);

        let beta_body = api
            .get_project_deserialized_common(beta_project_id, ADMIN_USER_PAT)
            .await;
        assert_eq!(beta_body.categories, alpha_body.categories);
        assert_eq!(
            beta_body.additional_categories,
            alpha_body.additional_categories,
        );
    })
    .await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_links() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;
            let alpha_project_id: &str =
                &test_env.dummy.project_alpha.project_id;
            let beta_project_id: &str = &test_env.dummy.project_beta.project_id;

            // Sets links for issue, source, wiki, and patreon for all projects
            // The first loop, sets issue, the second, clears it for all projects.
            for issues in [Some("https://www.issues.com"), None] {
                let resp = api
                    .edit_project_bulk(
                        &[alpha_project_id, beta_project_id],
                        json!({
                            "link_urls": {
                                "issues": issues,
                                "wiki": "https://wiki.com",
                                "patreon": "https://patreon.com",
                            },
                        }),
                        ADMIN_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::NO_CONTENT);

                let alpha_body = api
                    .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
                    .await;
                if let Some(issues) = issues {
                    assert_eq!(alpha_body.link_urls.len(), 3);
                    assert_eq!(alpha_body.link_urls["issues"].url, issues);
                } else {
                    assert_eq!(alpha_body.link_urls.len(), 2);
                    assert!(!alpha_body.link_urls.contains_key("issues"));
                }
                assert_eq!(
                    alpha_body.link_urls["wiki"].url,
                    "https://wiki.com"
                );
                assert_eq!(
                    alpha_body.link_urls["patreon"].url,
                    "https://patreon.com"
                );

                let beta_body = api
                    .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
                    .await;
                assert_eq!(beta_body.categories, alpha_body.categories);
                assert_eq!(
                    beta_body.additional_categories,
                    alpha_body.additional_categories,
                );
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_patch_project_v3() {
    with_test_environment(Some(8), |test_env: TestEnvironment<ApiV3>| async move {
        let alpha_project_id = &test_env.dummy.project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        let api = &test_env.api;
        // TODO: This should be a separate test from v3
        // - only a couple of these fields are v3-specific
        // once we have permissions/scope tests setup to not just take closures, we can split this up

        // For each permission covered by EDIT_DETAILS, ensure the permission is required
        let edit_details = ProjectPermissions::EDIT_DETAILS;
        let test_pairs = [
            // Body, status, requested_status tested separately
            ("slug", json!("")), // generated in the test to not collide slugs
            ("name", json!("randomname")),
            ("description", json!("randomdescription")),
            ("categories", json!(["combat", "economy"])),
            ("additional_categories", json!(["decoration"])),
            (
                "links",
                json!({
                    "issues": "https://issues.com",
                    "source": "https://source.com",
                }),
            ),
            ("license_id", json!("MIT")),
        ];

        futures::stream::iter(test_pairs)
            .map(|(key, value)| {
                let test_env = test_env.clone();
                async move {
                    let req_gen = |ctx: PermissionsTestContext| {
                        let value = value.clone();
                        async move {
                            api.edit_project(
                                &ctx.project_id.unwrap(),
                                json!({
                                    key: if key == "slug" {
                                        json!(generate_random_name("randomslug"))
                                    } else {
                                        value.clone()
                                    },
                                }),
                                ctx.test_pat.as_deref(),
                            )
                            .await
                        }
                    };
                    PermissionsTest::new(&test_env)
                        .simple_project_permissions_test(edit_details, req_gen)
                        .await
                        .into_iter();
                }
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

        // Test with status and requested_status
        // This requires a project with a version, so we use alpha_project_id
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "status": "private",
                    "requested_status": "private",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Bulk patch projects
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_bulk(
                &[&ctx.project_id.unwrap()],
                json!({
                    "name": "randomname",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Edit body
        // Cannot bulk edit body
        let edit_body = ProjectPermissions::EDIT_BODY;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "description": "new description!",
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(edit_body, req_gen)
            .await
            .unwrap();
    })
    .await;
}

// TODO: Project scheduling has been temporarily disabled, so this test is disabled as well
// #[actix_rt::test]
// async fn permissions_schedule() {
//     with_test_environment(None, |test_env : TestEnvironment<ApiV3>| async move {
//         let DummyProjectAlpha {
//             project_id: alpha_project_id,
//             team_id: alpha_team_id,
//             ..
//         } = &test_env.dummy.project_alpha;
//         let DummyProjectBeta {
//             project_id: beta_project_id,
//             version_id: beta_version_id,
//             team_id: beta_team_id,
//             ..
//         } = &test_env.dummy.project_beta;

//         let edit_details = ProjectPermissions::EDIT_DETAILS;
//         let api = &test_env.api;

//         // Approve beta version as private so we can schedule it
//         let resp = api
//             .edit_version(
//                 beta_version_id,
//                 json!({
//                     "status": "unlisted"
//                 }),
//                 MOD_USER_PAT,
//             )
//             .await;
//         assert_status!(&resp, StatusCode::NO_CONTENT);

//         // Schedule version
//         let req_gen = |ctx: PermissionsTestContext| async move {
//             api.schedule_version(
//                 beta_version_id,
//                 "archived",
//                 Utc::now() + Duration::days(1),
//                 ctx.test_pat.as_deref(),
//             )
//             .await
//         };
//         PermissionsTest::new(&test_env)
//             .with_existing_project(beta_project_id, beta_team_id)
//             .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
//             .simple_project_permissions_test(edit_details, req_gen)
//             .await
//             .unwrap();
//     }).await
// }

// Not covered by PATCH /project
#[actix_rt::test]
async fn permissions_edit_details() {
    with_test_environment_all(Some(10), |test_env| async move {
        let DummyProjectAlpha {
            project_id: alpha_project_id,
            team_id: alpha_team_id,
            ..
        } = &test_env.dummy.project_alpha;

        let edit_details = ProjectPermissions::EDIT_DETAILS;
        let api = &test_env.api;

        // Icon edit
        // Uses alpha project to delete this icon
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_icon(
                &ctx.project_id.unwrap(),
                Some(DummyImage::SmallIcon.get_icon_data()),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Icon delete
        // Uses alpha project to delete added icon
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project_icon(
                &ctx.project_id.unwrap(),
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Add gallery item
        // Uses alpha project to add gallery item so we can get its url
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_gallery_item(
                &ctx.project_id.unwrap(),
                DummyImage::SmallIcon.get_icon_data(),
                true,
                None,
                None,
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();
        // Get project, as we need the gallery image url
        let resp = api.get_project(alpha_project_id, USER_USER_PAT).await;
        let project: serde_json::Value = test::read_body_json(resp).await;
        let gallery_url = project["gallery"][0]["url"].as_str().unwrap();

        // Edit gallery item
        // Uses alpha project to edit gallery item
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_gallery_item(
                &ctx.project_id.unwrap(),
                gallery_url,
                vec![("description".to_string(), "new caption!".to_string())]
                    .into_iter()
                    .collect(),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();

        // Remove gallery item
        // Uses alpha project to remove gallery item
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_gallery_item(
                &ctx.project_id.unwrap(),
                gallery_url,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_details, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_upload_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let alpha_project_id = &test_env.dummy.project_alpha.project_id;
            let alpha_version_id = &test_env.dummy.project_alpha.version_id;
            let alpha_team_id = &test_env.dummy.project_alpha.team_id;
            let alpha_file_hash = &test_env.dummy.project_alpha.file_hash;

            let api = &test_env.api;

            let upload_version = ProjectPermissions::UPLOAD_VERSION;
            // Upload version with basic-mod.jar
            let req_gen = |ctx: PermissionsTestContext| async move {
                let project_id = ctx.project_id.unwrap();
                let project_id = ProjectId(parse_base62(&project_id).unwrap());
                api.add_public_version(
                    project_id,
                    "1.0.0",
                    TestFile::BasicMod,
                    None,
                    None,
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .simple_project_permissions_test(upload_version, req_gen)
                .await
                .unwrap();

            // Upload file to existing version
            // Uses alpha project, as it has an existing version
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.upload_file_to_version(
                    alpha_version_id,
                    &TestFile::BasicModDifferent,
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_project(alpha_project_id, alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_project_permissions_test(upload_version, req_gen)
                .await
                .unwrap();

            // Patch version
            // Uses alpha project, as it has an existing version
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.edit_version(
                    alpha_version_id,
                    json!({
                        "name": "Basic Mod",
                    }),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_project(alpha_project_id, alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_project_permissions_test(upload_version, req_gen)
                .await
                .unwrap();

            // Delete version file
            // Uses alpha project, as it has an existing version
            let delete_version = ProjectPermissions::DELETE_VERSION;
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.remove_version_file(
                    alpha_file_hash,
                    ctx.test_pat.as_deref(),
                )
                .await
            };

            PermissionsTest::new(&test_env)
                .with_existing_project(alpha_project_id, alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_project_permissions_test(delete_version, req_gen)
                .await
                .unwrap();

            // Delete version
            // Uses alpha project, as it has an existing version
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.remove_version(alpha_version_id, ctx.test_pat.as_deref())
                    .await
            };
            PermissionsTest::new(&test_env)
                .with_existing_project(alpha_project_id, alpha_team_id)
                .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
                .simple_project_permissions_test(delete_version, req_gen)
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_manage_invites() {
    // Add member, remove member, edit member
    with_test_environment_all(None, |test_env| async move {
        let alpha_project_id = &test_env.dummy.project_alpha.project_id;
        let alpha_team_id = &test_env.dummy.project_alpha.team_id;

        let api = &test_env.api;
        let manage_invites = ProjectPermissions::MANAGE_INVITES;

        // Add member
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.add_user_to_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                Some(ProjectPermissions::empty()),
                None,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // Edit member
        let edit_member = ProjectPermissions::EDIT_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_team_member(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                json!({
                    "permissions": 0,
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(edit_member, req_gen)
            .await
            .unwrap();

        // remove member
        // requires manage_invites if they have not yet accepted the invite
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(manage_invites, req_gen)
            .await
            .unwrap();

        // re-add member for testing
        let resp = api
            .add_user_to_team(
                alpha_team_id,
                MOD_USER_ID,
                Some(ProjectPermissions::empty()),
                None,
                ADMIN_USER_PAT,
            )
            .await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // Accept invite
        let resp = api.join_team(alpha_team_id, MOD_USER_PAT).await;
        assert_status!(&resp, StatusCode::NO_CONTENT);

        // remove existing member (requires remove_member)
        let remove_member = ProjectPermissions::REMOVE_MEMBER;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_from_team(
                &ctx.team_id.unwrap(),
                MOD_USER_ID,
                ctx.test_pat.as_deref(),
            )
            .await
        };

        PermissionsTest::new(&test_env)
            .with_existing_project(alpha_project_id, alpha_team_id)
            .with_user(FRIEND_USER_ID, FRIEND_USER_PAT, true)
            .simple_project_permissions_test(remove_member, req_gen)
            .await
            .unwrap();
    })
    .await;
}

#[actix_rt::test]
async fn permissions_delete_project() {
    // Add member, remove member, edit member
    with_test_environment_all(None, |test_env| async move {
        let delete_project = ProjectPermissions::DELETE_PROJECT;
        let api = &test_env.api;
        // Delete project
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.remove_project(
                &ctx.project_id.unwrap(),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .simple_project_permissions_test(delete_project, req_gen)
            .await
            .unwrap();

        test_env.cleanup().await;
    })
    .await;
}

#[actix_rt::test]
async fn project_permissions_consistency_test() {
    with_test_environment_all(Some(10), |test_env| async move {
        // Test that the permissions are consistent with each other
        // For example, if we get the projectpermissions directly, from an organization's defaults, overridden, etc, they should all be correct & consistent
        let api = &test_env.api;
        // Full project permissions test with EDIT_DETAILS
        let success_permissions = ProjectPermissions::EDIT_DETAILS;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "categories": [],
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .full_project_permissions_test(success_permissions, req_gen)
            .await
            .unwrap();

        // We do a test with more specific permissions, to ensure that *exactly* the permissions at each step are as expected
        let success_permissions = ProjectPermissions::EDIT_DETAILS
            | ProjectPermissions::REMOVE_MEMBER
            | ProjectPermissions::DELETE_VERSION
            | ProjectPermissions::VIEW_PAYOUTS;
        let req_gen = |ctx: PermissionsTestContext| async move {
            api.edit_project(
                &ctx.project_id.unwrap(),
                json!({
                    "categories": [],
                }),
                ctx.test_pat.as_deref(),
            )
            .await
        };
        PermissionsTest::new(&test_env)
            .full_project_permissions_test(success_permissions, req_gen)
            .await
            .unwrap();
    })
    .await;
}

// TODO: Re-add this if we want to match v3 Projects structure to v3 Search Result structure, otherwise, delete
// #[actix_rt::test]
// async fn align_search_projects() {
//     // Test setup and dummy data
//     with_test_environment(Some(10), |test_env: TestEnvironment<ApiV3>| async move {
//         setup_search_projects(&test_env).await;

//         let api = &test_env.api;
//         let test_name = test_env.db.database_name.clone();

//         let projects = api
//             .search_deserialized(
//                 Some(&format!("\"&{test_name}\"")),
//                 Some(json!([["categories:fabric"]])),
//                 USER_USER_PAT,
//             )
//             .await;

//         for project in projects.hits {
//             let project_model = api
//                 .get_project(&project.id.to_string(), USER_USER_PAT)
//                 .await;
//             assert_status!(&project_model, StatusCode::OK);
//             let mut project_model: Project = test::read_body_json(project_model).await;

//             // Body/description is huge- don't store it in search, so it's StatusCode::OK if they differ here
//             // (Search should return "")
//             project_model.description = "".into();

//             let project_model = serde_json::to_value(project_model).unwrap();
//             let searched_project_serialized = serde_json::to_value(project).unwrap();
//             assert_eq!(project_model, searched_project_serialized);
//         }
//     })
//     .await
// }

#[actix_rt::test]
async fn projects_various_visibility() {
    // For testing the filter_visible_projects and is_visible_project
    with_test_environment(
        None,
        |env: common::environment::TestEnvironment<ApiV3>| async move {
            let DummyProjectAlpha {
                project_id: alpha_project_id,
                project_id_parsed: alpha_project_id_parsed,
                ..
            } = &env.dummy.project_alpha;
            let DummyProjectBeta {
                project_id: beta_project_id,
                project_id_parsed: beta_project_id_parsed,
                ..
            } = &env.dummy.project_beta;
            let DummyOrganizationZeta {
                organization_id: zeta_organization_id,
                team_id: zeta_team_id,
                ..
            } = &env.dummy.organization_zeta;

            // Invite friend to org zeta and accept it
            let resp = env
                .api
                .add_user_to_team(
                    zeta_team_id,
                    FRIEND_USER_ID,
                    Some(ProjectPermissions::empty()),
                    None,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);
            let resp = env.api.join_team(zeta_team_id, FRIEND_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let visible_pat_pairs = vec![
                (&alpha_project_id_parsed, USER_USER_PAT, StatusCode::OK),
                (&alpha_project_id_parsed, FRIEND_USER_PAT, StatusCode::OK),
                (&alpha_project_id_parsed, ENEMY_USER_PAT, StatusCode::OK),
                (&beta_project_id_parsed, USER_USER_PAT, StatusCode::OK),
                (
                    &beta_project_id_parsed,
                    FRIEND_USER_PAT,
                    StatusCode::NOT_FOUND,
                ),
                (
                    &beta_project_id_parsed,
                    ENEMY_USER_PAT,
                    StatusCode::NOT_FOUND,
                ),
            ];

            // Tests get_project, a route that uses is_visible_project
            for (project_id, pat, expected_status) in visible_pat_pairs {
                let resp =
                    env.api.get_project(&project_id.to_string(), pat).await;
                assert_status!(&resp, expected_status);
            }

            // Test get_user_projects, a route that uses filter_visible_projects
            let visible_pat_pairs = vec![
                (USER_USER_PAT, 2),
                (FRIEND_USER_PAT, 1),
                (ENEMY_USER_PAT, 1),
            ];
            for (pat, expected_count) in visible_pat_pairs {
                let projects = env
                    .api
                    .get_user_projects_deserialized_common(USER_USER_ID, pat)
                    .await;
                assert_eq!(projects.len(), expected_count);
            }

            // Add projects to org zeta
            let resp = env
                .api
                .organization_add_project(
                    zeta_organization_id,
                    alpha_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);
            let resp = env
                .api
                .organization_add_project(
                    zeta_organization_id,
                    beta_project_id,
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::OK);

            // Test get_project, a route that uses is_visible_project
            let visible_pat_pairs = vec![
                (&alpha_project_id_parsed, USER_USER_PAT, StatusCode::OK),
                (&alpha_project_id_parsed, FRIEND_USER_PAT, StatusCode::OK),
                (&alpha_project_id_parsed, ENEMY_USER_PAT, StatusCode::OK),
                (&beta_project_id_parsed, USER_USER_PAT, StatusCode::OK),
                (&beta_project_id_parsed, FRIEND_USER_PAT, StatusCode::OK),
                (
                    &beta_project_id_parsed,
                    ENEMY_USER_PAT,
                    StatusCode::NOT_FOUND,
                ),
            ];

            for (project_id, pat, expected_status) in visible_pat_pairs {
                let resp =
                    env.api.get_project(&project_id.to_string(), pat).await;
                assert_status!(&resp, expected_status);
            }

            // Test get_user_projects, a route that uses filter_visible_projects
            let visible_pat_pairs = vec![
                (USER_USER_PAT, 2),
                (FRIEND_USER_PAT, 2),
                (ENEMY_USER_PAT, 1),
            ];
            for (pat, expected_count) in visible_pat_pairs {
                let projects = env
                    .api
                    .get_projects(&[alpha_project_id, beta_project_id], pat)
                    .await;
                let projects: Vec<CommonProject> =
                    test::read_body_json(projects).await;
                assert_eq!(projects.len(), expected_count);
            }
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_thread_deleted_with_project() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV3>| async move {
            let api = &test_env.api;

            let alpha_project_id = &test_env.dummy.project_alpha.project_id;
            let alpha_thread_id = &test_env.dummy.project_alpha.thread_id;

            // Verify the thread exists initially
            let resp = api.get_thread(alpha_thread_id, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);

            // Write a message to the thread to confirm it's working
            let resp = api
                .write_to_thread(
                    alpha_thread_id,
                    "text",
                    "Test message before project deletion",
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Check that the thread exists before project deletion
            // Use a moderator PAT since moderation threads are not visible to users
            let resp = api.get_thread(alpha_thread_id, MOD_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);

            // Delete the project
            let resp =
                api.remove_project(alpha_project_id, USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Check that the thread still exists after project deletion
            // Also use mod PAT here
            let resp = api.get_thread(alpha_thread_id, MOD_USER_PAT).await;
            assert_status!(&resp, StatusCode::OK);
        },
    )
    .await;
}

// Route tests:
// TODO: Missing routes on projects
// TODO: using permissions/scopes, can we SEE projects existence that we are not allowed to? (ie 401 instead of 404)

// Permissions:
// TODO: permissions VIEW_PAYOUTS currently is unused. Add tests when it is used.
// TODO: permissions VIEW_ANALYTICS currently is unused. Add tests when it is used.
