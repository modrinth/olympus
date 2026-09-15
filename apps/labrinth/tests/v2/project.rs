use std::sync::Arc;

use crate::{
    assert_status,
    common::{
        api_common::{ApiProject, ApiVersion, AppendsOptionalPat},
        api_v2::{ApiV2, request_data::get_public_project_creation_data_json},
        database::{
            ADMIN_USER_PAT, FRIEND_USER_ID, FRIEND_USER_PAT, USER_USER_PAT,
            generate_random_name,
        },
        dummy_data::TestFile,
        environment::{TestEnvironment, with_test_environment},
        permissions::{PermissionsTest, PermissionsTestContext},
    },
};
use actix_http::StatusCode;
use actix_web::test;
use ariadne::ids::base62_impl::parse_base62;
use futures::StreamExt;
use hex::ToHex;
use itertools::Itertools;
use labrinth::models::ids::ProjectId;
use labrinth::{
    database::models::project_item::PROJECTS_SLUGS_NAMESPACE,
    models::teams::ProjectPermissions,
    util::actix::{AppendsMultipart, MultipartSegment, MultipartSegmentData},
};
use serde_json::json;
use sha1::Digest;

#[actix_rt::test]
async fn test_project_type_sanity() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            // Perform all other patch tests on both 'mod' and 'modpack'
            for (mod_or_modpack, slug, file) in [
                ("mod", "test-mod", TestFile::build_random_jar()),
                ("modpack", "test-modpack", TestFile::build_random_mrpack()),
            ] {
                // Create a modpack or mod
                // both are 'fabric' (but modpack is actually 'mrpack' behind the scenes, through v3,with fabric as a 'mrpack_loader')
                let (test_project, test_version) = api
                    .add_public_project(slug, Some(file), None, USER_USER_PAT)
                    .await;
                let test_project_slug = test_project.slug.as_ref().unwrap();

                // Check that the loader displays correctly as fabric from the version creation
                assert_eq!(test_project.loaders, vec!["fabric"]);
                assert_eq!(test_version[0].loaders, vec!["fabric"]);

                // Check that the project type is correct when getting the project
                let project = api
                    .get_project_deserialized(test_project_slug, USER_USER_PAT)
                    .await;
                assert_eq!(test_project.loaders, vec!["fabric"]);
                assert_eq!(project.project_type, mod_or_modpack);

                // Check that the project type is correct when getting the version
                let version = api
                    .get_version_deserialized(
                        &test_version[0].id.to_string(),
                        USER_USER_PAT,
                    )
                    .await;
                assert_eq!(
                    version.loaders.iter().map(|x| &x.0).collect_vec(),
                    vec!["fabric"]
                );

                // Edit the version loader to change it to 'forge'
                let resp = api
                    .edit_version(
                        &test_version[0].id.to_string(),
                        json!({
                            "loaders": ["forge"],
                        }),
                        USER_USER_PAT,
                    )
                    .await;
                assert_status!(&resp, StatusCode::NO_CONTENT);

                // Check that the project type is still correct when getting the project
                let project = api
                    .get_project_deserialized(test_project_slug, USER_USER_PAT)
                    .await;
                assert_eq!(project.project_type, mod_or_modpack);
                assert_eq!(project.loaders, vec!["forge"]);

                // Check that the project type is still correct when getting the version
                let version = api
                    .get_version_deserialized(
                        &test_version[0].id.to_string(),
                        USER_USER_PAT,
                    )
                    .await;
                assert_eq!(
                    version.loaders.iter().map(|x| &x.0).collect_vec(),
                    vec!["forge"]
                );
            }

            // As we get more complicated structures with as v3 continues to expand, and alpha/beta get more complicated, we should add more tests here,
            // to ensure that projects created with v3 routes are still valid and work with v3 routes.
        },
    )
    .await;
}

#[actix_rt::test]
async fn test_add_remove_project() {
    // Test setup and dummy data
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            // Generate test project data.
            let mut json_data = get_public_project_creation_data_json(
                "demo",
                Some(&TestFile::BasicMod),
            );

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
            let req = test::TestRequest::post()
                .uri("/v2/project")
                .append_pat(USER_USER_PAT)
                .set_multipart(vec![json_segment.clone(), file_segment.clone()])
                .to_request();
            let resp: actix_web::dev::ServiceResponse =
                test_env.call(req).await;
            assert_status!(&resp, StatusCode::OK);

            // Get the project we just made, and confirm that it's correct
            let project =
                api.get_project_deserialized("demo", USER_USER_PAT).await;
            assert!(project.versions.len() == 1);
            let uploaded_version_id = project.versions[0];

            // Checks files to ensure they were uploaded and correctly identify the file
            let hash = sha1::Sha1::digest(basic_mod_file.bytes())
                .encode_hex::<String>();
            let version = api
                .get_version_from_hash_deserialized(
                    &hash,
                    "sha1",
                    USER_USER_PAT,
                )
                .await;
            assert_eq!(version.id, uploaded_version_id);

            // Reusing with a different slug and the same file should fail
            // Even if that file is named differently
            let req = test::TestRequest::post()
                .uri("/v2/project")
                .append_pat(USER_USER_PAT)
                .set_multipart(vec![
                    json_diff_slug_file_segment.clone(), // Different slug, different file name
                    file_diff_name_segment.clone(), // Different file name, same content
                ])
                .to_request();

            let resp = test_env.call(req).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Reusing with the same slug and a different file should fail
            let req = test::TestRequest::post()
                .uri("/v2/project")
                .append_pat(USER_USER_PAT)
                .set_multipart(vec![
                    json_diff_file_segment.clone(), // Same slug, different file name
                    file_diff_name_content_segment.clone(), // Different file name, different content
                ])
                .to_request();

            let resp = test_env.call(req).await;
            assert_status!(&resp, StatusCode::BAD_REQUEST);

            // Different slug, different file should succeed
            let req = test::TestRequest::post()
                .uri("/v2/project")
                .append_pat(USER_USER_PAT)
                .set_multipart(vec![
                    json_diff_slug_file_segment.clone(), // Different slug, different file name
                    file_diff_name_content_segment.clone(), // Different file name, same content
                ])
                .to_request();

            let resp = test_env.call(req).await;
            assert_status!(&resp, StatusCode::OK);

            // Get
            let project =
                api.get_project_deserialized("demo", USER_USER_PAT).await;
            let id = project.id.to_string();

            // Remove the project
            let resp = test_env.api.remove_project("demo", USER_USER_PAT).await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            // Confirm that the project is gone from the cache
            let mut redis_conn =
                test_env.db.redis_pool.connect().await.unwrap();
            let slug_key =
                redis_conn.key().entity(PROJECTS_SLUGS_NAMESPACE, "demo");
            let id_key = redis_conn.key().entity(PROJECTS_SLUGS_NAMESPACE, &id);
            assert_eq!(
                redis_conn
                    .get(&slug_key)
                    .await
                    .unwrap()
                    .map(|x| x.parse::<i64>().unwrap()),
                None
            );
            assert_eq!(
                redis_conn
                    .get(&id_key)
                    .await
                    .unwrap()
                    .map(|x| x.parse::<i64>().unwrap()),
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
async fn permissions_upload_version() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let alpha_project_id = &test_env.dummy.project_alpha.project_id;
            let alpha_version_id = &test_env.dummy.project_alpha.version_id;
            let alpha_team_id = &test_env.dummy.project_alpha.team_id;
            let alpha_file_hash = &test_env.dummy.project_alpha.file_hash;

            let api = &test_env.api;
            let basic_mod_different_file = TestFile::BasicModDifferent;
            let upload_version = ProjectPermissions::UPLOAD_VERSION;

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
            let file_ref = Arc::new(basic_mod_different_file);
            let req_gen = |ctx: PermissionsTestContext| {
                let file_ref = file_ref.clone();
                async move {
                    api.upload_file_to_version(
                        alpha_version_id,
                        &file_ref,
                        ctx.test_pat.as_deref(),
                    )
                    .await
                }
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
async fn invalid_review_submission_returns_validation_error() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
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

            let response = api
                .edit_project(
                    project_slug,
                    json!({
                        "body": "",
                        "status": "processing",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&response, StatusCode::BAD_REQUEST);

            let error: serde_json::Value = test::read_body_json(response).await;
            assert_eq!(
                error["description"],
                "resolve required project validation messages before saving"
            );
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_patch_v2() {
    // Hits V3-specific patchable fields
    // Other fields are tested in test_patch_project (the v2 version of that test)
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            let alpha_project_slug = &test_env.dummy.project_alpha.project_slug;

            // Successful request to patch many fields.
            let resp = api
                .edit_project(
                    alpha_project_slug,
                    json!({
                        "client_side": "unsupported",
                        "server_side": "required",
                    }),
                    USER_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let project = api
                .get_project_deserialized(alpha_project_slug, USER_USER_PAT)
                .await;

            // Note: the original V2 value of this was "optional",
            // but Required/Optional is no longer a carried combination in v3, as the changes made were lossy.
            // Now, the test Required/Unsupported combination is tested instead.
            // Setting Required/Optional in v2 will not work, this is known and accepted.
            assert_eq!(project.client_side.as_str(), "unsupported");
            assert_eq!(project.server_side.as_str(), "required");
        },
    )
    .await;
}

#[actix_rt::test]
async fn permissions_patch_project_v2() {
    with_test_environment(
        Some(8),
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;

            // For each permission covered by EDIT_DETAILS, ensure the permission is required
            let edit_details = ProjectPermissions::EDIT_DETAILS;
            let test_pairs = [
                ("description", json!("description")),
                ("issues_url", json!("https://issues.com")),
                ("source_url", json!("https://source.com")),
                ("wiki_url", json!("https://wiki.com")),
                (
                    "donation_urls",
                    json!([{
                        "id": "paypal",
                        "platform": "Paypal",
                        "url": "https://paypal.com"
                    }]),
                ),
                ("discord_url", json!("https://discord.com")),
            ];

            futures::stream::iter(test_pairs)
                .map(|(key, value)| {
                    let test_env = test_env.clone();
                    async move {
                        let req_gen = async |ctx: PermissionsTestContext| {
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
                        };
                        PermissionsTest::new(&test_env)
                            .simple_project_permissions_test(
                                edit_details,
                                req_gen,
                            )
                            .await
                            .unwrap();
                    }
                })
                .buffer_unordered(4)
                .collect::<Vec<_>>()
                .await;

            // Edit body
            // Cannot bulk edit body
            let edit_body = ProjectPermissions::EDIT_BODY;
            let req_gen = |ctx: PermissionsTestContext| async move {
                api.edit_project(
                    &ctx.project_id.unwrap(),
                    json!({
                        "body": "new body!", // new body
                    }),
                    ctx.test_pat.as_deref(),
                )
                .await
            };
            PermissionsTest::new(&test_env)
                .simple_project_permissions_test(edit_body, req_gen)
                .await
                .unwrap();
        },
    )
    .await;
}

#[actix_rt::test]
pub async fn test_bulk_edit_links() {
    with_test_environment(
        None,
        |test_env: TestEnvironment<ApiV2>| async move {
            let api = &test_env.api;
            let alpha_project_id: &str =
                &test_env.dummy.project_alpha.project_id;
            let beta_project_id: &str = &test_env.dummy.project_beta.project_id;

            let resp = api
                .edit_project_bulk(
                    &[alpha_project_id, beta_project_id],
                    json!({
                        "issues_url": "https://github.com/modrinth/code/issues",
                        "donation_urls": [
                            {
                                "id": "patreon",
                                "platform": "Patreon",
                                "url": "https://www.patreon.com/my_user"
                            }
                        ],
                    }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let alpha_body = api
                .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = alpha_body.donation_urls.unwrap();
            assert_eq!(donation_urls.len(), 1);
            assert_eq!(donation_urls[0].url, "https://www.patreon.com/my_user");
            assert_eq!(
                alpha_body.issues_url,
                Some("https://github.com/modrinth/code/issues".to_string())
            );
            assert_eq!(alpha_body.discord_url, None);

            let beta_body = api
                .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = beta_body.donation_urls.unwrap();
            assert_eq!(donation_urls.len(), 1);
            assert_eq!(donation_urls[0].url, "https://www.patreon.com/my_user");
            assert_eq!(
                beta_body.issues_url,
                Some("https://github.com/modrinth/code/issues".to_string())
            );
            assert_eq!(beta_body.discord_url, None);

            let resp = api
                .edit_project_bulk(
                    &[alpha_project_id, beta_project_id],
                    json!({
                        "discord_url": "https://discord.gg/modrinth",
                        "issues_url": null,
                        "add_donation_urls": [
                            {
                                "id": "bmac",
                                "platform": "Buy Me a Coffee",
                                "url": "https://www.buymeacoffee.com/my_user"
                            }
                        ],
                    }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let alpha_body = api
                .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = alpha_body
                .donation_urls
                .unwrap()
                .into_iter()
                .sorted_by_key(|x| x.id.clone())
                .collect_vec();
            assert_eq!(donation_urls.len(), 2);
            assert_eq!(
                donation_urls[0].url,
                "https://www.buymeacoffee.com/my_user"
            );
            assert_eq!(donation_urls[1].url, "https://www.patreon.com/my_user");
            assert_eq!(alpha_body.issues_url, None);
            assert_eq!(
                alpha_body.discord_url,
                Some("https://discord.gg/modrinth".to_string())
            );

            let beta_body = api
                .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = beta_body
                .donation_urls
                .unwrap()
                .into_iter()
                .sorted_by_key(|x| x.id.clone())
                .collect_vec();
            assert_eq!(donation_urls.len(), 2);
            assert_eq!(
                donation_urls[0].url,
                "https://www.buymeacoffee.com/my_user"
            );
            assert_eq!(donation_urls[1].url, "https://www.patreon.com/my_user");
            assert_eq!(beta_body.issues_url, None);
            assert_eq!(
                beta_body.discord_url,
                Some("https://discord.gg/modrinth".to_string())
            );

            let resp = api
                .edit_project_bulk(
                    &[alpha_project_id, beta_project_id],
                    json!({
                        "donation_urls": [
                            {
                                "id": "patreon",
                                "platform": "Patreon",
                                "url": "https://www.patreon.com/my_user"
                            },
                            {
                                "id": "ko-fi",
                                "platform": "Ko-fi",
                                "url": "https://www.ko-fi.com/my_user"
                            }
                        ],
                        "add_donation_urls": [
                            {
                                "id": "paypal",
                                "platform": "PayPal",
                                "url": "https://www.paypal.com/my_user"
                            }
                        ],
                        "remove_donation_urls": [
                            {
                                "id": "ko-fi",
                                "platform": "Ko-fi",
                                "url": "https://www.ko-fi.com/my_user"
                            }
                        ],
                    }),
                    ADMIN_USER_PAT,
                )
                .await;
            assert_status!(&resp, StatusCode::NO_CONTENT);

            let alpha_body = api
                .get_project_deserialized(alpha_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = alpha_body
                .donation_urls
                .unwrap()
                .into_iter()
                .sorted_by_key(|x| x.id.clone())
                .collect_vec();
            assert_eq!(donation_urls.len(), 2);
            assert_eq!(donation_urls[0].url, "https://www.patreon.com/my_user");
            assert_eq!(donation_urls[1].url, "https://www.paypal.com/my_user");

            let beta_body = api
                .get_project_deserialized(beta_project_id, ADMIN_USER_PAT)
                .await;
            let donation_urls = beta_body
                .donation_urls
                .unwrap()
                .into_iter()
                .sorted_by_key(|x| x.id.clone())
                .collect_vec();
            assert_eq!(donation_urls.len(), 2);
            assert_eq!(donation_urls[0].url, "https://www.patreon.com/my_user");
            assert_eq!(donation_urls[1].url, "https://www.paypal.com/my_user");
        },
    )
    .await;
}
