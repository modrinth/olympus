//! Theseus profile management interface

use crate::event::emit::{
    emit_loading, init_loading, loading_try_for_each_concurrent,
};
use crate::event::LoadingBarType;
use crate::pack::install_from::{
    EnvType, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::prelude::{JavaVersion, ProfilePathId, ProjectPathId};
use crate::state::{
    Credentials, InnerProjectPathUnix, ProjectMetadata, SideType,
};

use crate::util::fetch;
use crate::util::io::{self, IOError};
use crate::{
    event::{emit::emit_profile, ProfilePayloadType},
    state::MinecraftChild,
};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use async_zip::tokio::write::ZipFileWriter;
use async_zip::{Compression, ZipEntryBuilder};
use serde_json::json;

use std::collections::{HashMap, HashSet};

use std::iter::FromIterator;
use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::io::AsyncReadExt;
use tokio::{fs::File, process::Command, sync::RwLock};

pub mod create;
pub mod update;

/// Remove a profile
#[tracing::instrument]
pub async fn remove(path: &ProfilePathId) -> crate::Result<()> {
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    if let Some(profile) = profiles.remove(path).await? {
        emit_profile(
            profile.uuid,
            path,
            &profile.metadata.name,
            ProfilePayloadType::Removed,
        )
        .await?;
    }

    Ok(())
}

/// Get a profile by relative path (or, name)
#[tracing::instrument]
pub async fn get(
    path: &ProfilePathId,
    clear_projects: Option<bool>,
) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    let mut profile = profiles.0.get(path).cloned();

    if clear_projects.unwrap_or(false) {
        if let Some(profile) = &mut profile {
            profile.projects = HashMap::new();
        }
    }

    Ok(profile)
}

/// Get a profile by uuid
#[tracing::instrument]
pub async fn get_by_uuid(
    uuid: uuid::Uuid,
    clear_projects: Option<bool>,
) -> crate::Result<Option<Profile>> {
    let state = State::get().await?;

    let profiles = state.profiles.read().await;
    let mut profile = profiles.0.values().find(|x| x.uuid == uuid).cloned();

    if clear_projects.unwrap_or(false) {
        if let Some(profile) = &mut profile {
            profile.projects = HashMap::new();
        }
    }

    Ok(profile)
}

/// Get profile's full path in the filesystem
#[tracing::instrument]
pub async fn get_full_path(path: &ProfilePathId) -> crate::Result<PathBuf> {
    let _ = get(path, Some(true)).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to get the full path of a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;
    let full_path = io::canonicalize(path.get_full_path().await?)?;
    Ok(full_path)
}

/// Get mod's full path in the filesystem
#[tracing::instrument]
pub async fn get_mod_full_path(
    profile_path: &ProfilePathId,
    project_path: &ProjectPathId,
) -> crate::Result<PathBuf> {
    if get(profile_path, Some(true)).await?.is_some() {
        let full_path = io::canonicalize(
            project_path.get_full_path(profile_path.clone()).await?,
        )?;
        return Ok(full_path);
    }

    Err(crate::ErrorKind::OtherError(format!(
        "Tried to get the full path of a nonexistent or unloaded project at path {}!",
        project_path.get_full_path(profile_path.clone()).await?.display()
    ))
    .into())
}

/// Edit a profile using a given asynchronous closure
pub async fn edit<Fut>(
    path: &ProfilePathId,
    action: impl Fn(&mut Profile) -> Fut,
) -> crate::Result<()>
where
    Fut: Future<Output = crate::Result<()>>,
{
    let state = State::get().await?;
    let mut profiles = state.profiles.write().await;

    match profiles.0.get_mut(path) {
        Some(ref mut profile) => {
            action(profile).await?;

            emit_profile(
                profile.uuid,
                path,
                &profile.metadata.name,
                ProfilePayloadType::Edited,
            )
            .await?;

            Ok(())
        }
        None => Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error()),
    }
}

/// Edits a profile's icon
pub async fn edit_icon(
    path: &ProfilePathId,
    icon_path: Option<&Path>,
) -> crate::Result<()> {
    let state = State::get().await?;

    let res = if let Some(icon) = icon_path {
        let bytes = io::read(icon).await?;

        let mut profiles = state.profiles.write().await;

        match profiles.0.get_mut(path) {
            Some(ref mut profile) => {
                profile
                    .set_icon(
                        &state.directories.caches_dir(),
                        &state.io_semaphore,
                        bytes::Bytes::from(bytes),
                        &icon.to_string_lossy(),
                    )
                    .await?;

                emit_profile(
                    profile.uuid,
                    path,
                    &profile.metadata.name,
                    ProfilePayloadType::Edited,
                )
                .await?;
                Ok(())
            }
            None => {
                Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
                    .as_error())
            }
        }
    } else {
        edit(path, |profile| {
            profile.metadata.icon = None;
            async { Ok(()) }
        })
        .await?;
        State::sync().await?;

        Ok(())
    };
    State::sync().await?;
    res
}

// Gets the optimal JRE key for the given profile, using Daedalus
// Generally this would be used for profile_create, to get the optimal JRE key
// this can be overwritten by the user a profile-by-profile basis
pub async fn get_optimal_jre_key(
    path: &ProfilePathId,
) -> crate::Result<Option<JavaVersion>> {
    let state = State::get().await?;

    if let Some(profile) = get(path, None).await? {
        let metadata = state.metadata.read().await;

        // Fetch version info from stored profile game_version
        let version = metadata
            .minecraft
            .versions
            .iter()
            .find(|it| it.id == profile.metadata.game_version)
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Invalid or unknown Minecraft version: {}",
                    profile.metadata.game_version
                ))
            })?;

        // Get detailed manifest info from Daedalus
        let version_info = crate::launcher::download::download_version_info(
            &state,
            version,
            profile.metadata.loader_version.as_ref(),
            None,
            None,
        )
        .await?;

        let version = crate::launcher::get_java_version_from_profile(
            &profile,
            &version_info,
        )
        .await?;

        Ok(version)
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error())
    }
}

/// Get a copy of the profile set
#[tracing::instrument]
pub async fn list(
    clear_projects: Option<bool>,
) -> crate::Result<HashMap<ProfilePathId, Profile>> {
    let state = State::get().await?;
    let profiles = state.profiles.read().await;
    Ok(profiles
        .0
        .clone()
        .into_iter()
        .map(|mut x| {
            if clear_projects.unwrap_or(false) {
                x.1.projects = HashMap::new();
            }

            x
        })
        .collect())
}

/// Installs/Repairs a profile
#[tracing::instrument]
pub async fn install(path: &ProfilePathId, force: bool) -> crate::Result<()> {
    if let Some(profile) = get(path, None).await? {
        crate::launcher::install_minecraft(&profile, None, force).await?;
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(path.to_string())
            .as_error());
    }
    State::sync().await?;
    Ok(())
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_all_projects(
    profile_path: &ProfilePathId,
) -> crate::Result<HashMap<ProjectPathId, ProjectPathId>> {
    if let Some(profile) = get(profile_path, None).await? {
        let loading_bar = init_loading(
            LoadingBarType::ProfileUpdate {
                profile_path: profile.get_profile_full_path().await?,
                profile_name: profile.metadata.name.clone(),
            },
            100.0,
            "Updating profile",
        )
        .await?;

        let keys = profile
            .projects
            .into_iter()
            .filter(|(_, project)| {
                matches!(
                    &project.metadata,
                    ProjectMetadata::Modrinth {
                        update_version: Some(_),
                        ..
                    }
                )
            })
            .map(|x| x.0)
            .collect::<Vec<_>>();
        let len = keys.len();

        let map = Arc::new(RwLock::new(HashMap::new()));

        use futures::StreamExt;
        loading_try_for_each_concurrent(
            futures::stream::iter(keys).map(Ok::<ProjectPathId, crate::Error>),
            None,
            Some(&loading_bar),
            100.0,
            len,
            None,
            |project| async {
                let map = map.clone();

                async move {
                    let new_path =
                        update_project(profile_path, &project, Some(true))
                            .await?;

                    map.write().await.insert(project, new_path);

                    Ok(())
                }
                .await
            },
        )
        .await?;

        emit_profile(
            profile.uuid,
            profile_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(Arc::try_unwrap(map).unwrap().into_inner())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Updates a project to the latest version
/// Uses and returns the relative path to the project
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn update_project(
    profile_path: &ProfilePathId,
    project_path: &ProjectPathId,
    skip_send_event: Option<bool>,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        if let Some(project) = profile.projects.get(project_path) {
            if let ProjectMetadata::Modrinth {
                update_version: Some(update_version),
                ..
            } = &project.metadata
            {
                let (path, new_version) = profile
                    .add_project_version(update_version.id.clone())
                    .await?;

                if project.disabled {
                    profile.toggle_disable_project(&path).await?;
                }

                if path != project_path.clone() {
                    profile.remove_project(project_path, Some(true)).await?;
                }

                let state = State::get().await?;
                let mut profiles = state.profiles.write().await;
                if let Some(profile) = profiles.0.get_mut(profile_path) {
                    let value = profile.projects.remove(project_path);
                    if let Some(mut project) = value {
                        if let ProjectMetadata::Modrinth {
                            ref mut version,
                            ref mut update_version,
                            ..
                        } = project.metadata
                        {
                            *version = Box::new(new_version);
                            *update_version = None;
                        }
                        profile.projects.insert(path.clone(), project);
                    }
                }
                drop(profiles);

                if !skip_send_event.unwrap_or(false) {
                    emit_profile(
                        profile.uuid,
                        profile_path,
                        &profile.metadata.name,
                        ProfilePayloadType::Edited,
                    )
                    .await?;
                    State::sync().await?;
                }

                return Ok(path);
            }
        }

        Err(crate::ErrorKind::InputError(
            "This project cannot be updated!".to_string(),
        )
        .as_error())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Add a project from a version
/// Returns the relative path to the project as a ProjectPathId
#[tracing::instrument]
pub async fn add_project_from_version(
    profile_path: &ProfilePathId,
    version_id: String,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        let (project_path, _) = profile.add_project_version(version_id).await?;

        emit_profile(
            profile.uuid,
            profile_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        Ok(project_path)
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Add a project from an FS path
/// Uses and returns the relative path to the project as a ProjectPathId
#[tracing::instrument]
pub async fn add_project_from_path(
    profile_path: &ProfilePathId,
    path: &Path,
    project_type: Option<String>,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        let file = io::read(path).await?;
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let path = profile
            .add_project_bytes(
                &file_name,
                bytes::Bytes::from(file),
                project_type.and_then(|x| serde_json::from_str(&x).ok()),
            )
            .await?;

        emit_profile(
            profile.uuid,
            profile_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(path)
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Toggle whether a project is disabled or not
/// Project path should be relative to the profile
/// returns the new state, relative to the profile
#[tracing::instrument]
pub async fn toggle_disable_project(
    profile_path: &ProfilePathId,
    project: &ProjectPathId,
) -> crate::Result<ProjectPathId> {
    if let Some(profile) = get(profile_path, None).await? {
        let res = profile.toggle_disable_project(project).await?;

        emit_profile(
            profile.uuid,
            profile_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(res)
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Remove a project from a profile
/// Uses and returns the relative path to the project
#[tracing::instrument]
pub async fn remove_project(
    profile_path: &ProfilePathId,
    project: &ProjectPathId,
) -> crate::Result<()> {
    if let Some(profile) = get(profile_path, None).await? {
        profile.remove_project(project, None).await?;

        emit_profile(
            profile.uuid,
            profile_path,
            &profile.metadata.name,
            ProfilePayloadType::Edited,
        )
        .await?;
        State::sync().await?;

        Ok(())
    } else {
        Err(
            crate::ErrorKind::UnmanagedProfileError(profile_path.to_string())
                .as_error(),
        )
    }
}

/// Exports the profile to a Modrinth-formatted .mrpack file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument(skip_all)]
#[theseus_macros::debug_pin]
pub async fn export_mrpack(
    profile_path: &ProfilePathId,
    export_path: PathBuf,
    included_export_candidates: Vec<String>, // which folders/files to include in the export
    version_id: Option<String>,
    description: Option<String>,
    _name: Option<String>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let io_semaphore = state.io_semaphore.0.read().await;
    let _permit: tokio::sync::SemaphorePermit = io_semaphore.acquire().await?;
    let profile = get(profile_path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent or unloaded profile at path {}!",
            profile_path
        ))
    })?;

    // remove .DS_Store files from included_export_candidates
    let included_export_candidates = included_export_candidates
        .into_iter()
        .filter(|x| {
            if let Some(f) = PathBuf::from(x).file_name() {
                if f.to_string_lossy().starts_with(".DS_Store") {
                    return false;
                }
            }
            true
        })
        .collect::<Vec<_>>();

    let profile_base_path = &profile.get_profile_full_path().await?;

    let mut file = File::create(&export_path)
        .await
        .map_err(|e| IOError::with_path(e, &export_path))?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    // Create mrpack json configuration file
    let version_id = version_id.unwrap_or("1.0.0".to_string());
    let mut packfile =
        create_mrpack_json(&profile, version_id, description).await?;
    let included_candidates_set =
        HashSet::<_>::from_iter(included_export_candidates.iter());
    packfile.files.retain(|f| {
        included_candidates_set.contains(&f.path.get_topmost_two_components())
    });

    // Build vec of all files in the folder
    let mut path_list = Vec::new();
    add_all_recursive_folder_paths(profile_base_path, &mut path_list).await?;

    // Initialize loading bar
    let loading_bar = init_loading(
        LoadingBarType::ZipExtract {
            profile_path: profile.get_profile_full_path().await?,
            profile_name: profile.metadata.name.clone(),
        },
        path_list.len() as f64,
        "Exporting profile to .mrpack",
    )
    .await?;

    // Iterate over every file in the folder
    // Every file that is NOT in the config file is added to the zip, in overrides
    for path in path_list {
        emit_loading(&loading_bar, 1.0, None).await?;

        let relative_path = ProjectPathId::from_fs_path(&path)
            .await?
            .get_inner_path_unix();
        if packfile.files.iter().any(|f| f.path == relative_path)
            || !included_candidates_set
                .contains(&relative_path.get_topmost_two_components())
        {
            continue;
        }

        // File is not in the config file, add it to the .mrpack zip
        if path.is_file() {
            let mut file = File::open(&path)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .await
                .map_err(|e| IOError::with_path(e, &path))?;
            let builder = ZipEntryBuilder::new(
                format!("overrides/{relative_path}").into(),
                Compression::Deflate,
            );
            writer.write_entry_whole(builder, &data).await?;
        }
    }

    // Add modrinth json to the zip
    let data = serde_json::to_vec_pretty(&packfile)?;
    let builder = ZipEntryBuilder::new(
        "modrinth.index.json".to_string().into(),
        Compression::Deflate,
    );
    writer.write_entry_whole(builder, &data).await?;

    writer.close().await?;

    Ok(())
}

// Given a folder path, populate a Vec of all the subfolders and files, at most 2 layers deep
// profile
// -- folder1
// -- folder2
//    -- innerfolder
//       -- innerfile
//    -- folder2file
// -- file1
// => [folder1, folder2/innerfolder, folder2/folder2file, file1]
#[tracing::instrument]
pub async fn get_pack_export_candidates(
    profile_path: &ProfilePathId,
) -> crate::Result<Vec<InnerProjectPathUnix>> {
    // First, get a dummy mrpack json for the files within
    let profile: Profile = get(profile_path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to export a nonexistent or unloaded profile at path {}!",
            profile_path
        ))
    })?;

    let mut path_list: Vec<InnerProjectPathUnix> = Vec::new();

    let profile_base_dir = profile.get_profile_full_path().await?;
    let mut read_dir = io::read_dir(&profile_base_dir).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &profile_base_dir))?
    {
        let path: PathBuf = entry.path();
        if path.is_dir() {
            // Two layers of files/folders if its a folder
            let mut read_dir = io::read_dir(&path).await?;
            while let Some(entry) = read_dir
                .next_entry()
                .await
                .map_err(|e| IOError::with_path(e, &profile_base_dir))?
            {
                let path: PathBuf = entry.path();
                if let Ok(project_path) =
                    ProjectPathId::from_fs_path(&path).await
                {
                    path_list.push(project_path.get_inner_path_unix());
                }
            }
        } else {
            // One layer of files/folders if its a file
            if let Ok(project_path) = ProjectPathId::from_fs_path(&path).await {
                path_list.push(project_path.get_inner_path_unix());
            }
        }
    }
    Ok(path_list)
}

/// Run Minecraft using a profile and the default credentials, logged in credentials,
/// failing with an error if no credentials are available
#[tracing::instrument]
pub async fn run(
    path: &ProfilePathId,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;

    // Get default account and refresh credentials (preferred way to log in)
    let default_account = {
        let mut write = state.users.write().await;

        write
            .get_default_credential()
            .await?
            .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?
    };

    run_credentials(path, &default_account).await
}

/// Run Minecraft using a profile, and credentials for authentication
/// Returns Arc pointer to RwLock to Child
#[tracing::instrument(skip(credentials))]
#[theseus_macros::debug_pin]
pub async fn run_credentials(
    path: &ProfilePathId,
    credentials: &Credentials,
) -> crate::Result<Arc<RwLock<MinecraftChild>>> {
    let state = State::get().await?;
    let settings = state.settings.read().await;
    let profile = get(path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to run a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;

    let java_args = profile
        .java
        .as_ref()
        .and_then(|it| it.extra_arguments.as_ref())
        .unwrap_or(&settings.custom_java_args);

    let pre_launch_hooks =
        &profile.hooks.as_ref().unwrap_or(&settings.hooks).pre_launch;
    if let Some(hook) = pre_launch_hooks {
        // TODO: Improve Hook parameters
        
        let full_path = path.get_full_path().await?;
        let jre_key = get_optimal_jre_key(&path).await?;
        // The hook parameters are compatible with MultiMC-based and other launchers. See https://github.com/MultiMC/Launcher/wiki/Instance-settings#custom-commands
        let cmd_with_hook_parameters = hook
            .replace("$INST_NAME", profile.metadata.name.as_str())
            .replace("$INST_ID", profile.metadata.name.as_str())
            .replace("$INST_DIR", full_path.to_str().unwrap())
            .replace("$INST_MC_DIR", full_path.to_str().unwrap())
            .replace("$INST_JAVA", jre_key.unwrap().path.as_str())
            // TODO: Pass the args correctly
            .replace("$INST_JAVA_ARGS", java_args.join(" ").as_str());

        let mut cmd = cmd_with_hook_parameters.split(' ');
        if let Some(command) = cmd.next() {
            let result = Command::new(command)
                .args(&cmd.collect::<Vec<&str>>())
                .current_dir(&full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, &full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    let wrapper = profile
        .hooks
        .as_ref()
        .map_or(&settings.hooks.wrapper, |it| &it.wrapper);

    let memory = profile.memory.unwrap_or(settings.memory);
    let resolution = profile.resolution.unwrap_or(settings.game_resolution);

    let env_args = profile
        .java
        .as_ref()
        .and_then(|x| x.custom_env_args.as_ref())
        .unwrap_or(&settings.custom_env_args);

    // Post post exit hooks
    let post_exit_hook = profile
        .hooks
        .as_ref()
        .unwrap_or(&settings.hooks)
        .post_exit
        .clone();

    // Any options.txt settings that we want set, add here
    let mut mc_set_options: Vec<(String, String)> = vec![];
    if let Some(fullscreen) = profile.fullscreen {
        // Profile fullscreen setting takes priority
        mc_set_options.push(("fullscreen".to_string(), fullscreen.to_string()));
    } else if settings.force_fullscreen {
        // If global settings wants to force a fullscreen, do it
        mc_set_options.push(("fullscreen".to_string(), "true".to_string()));
    }

    let mc_process = crate::launcher::launch_minecraft(
        java_args,
        env_args,
        &mc_set_options,
        wrapper,
        &memory,
        &resolution,
        credentials,
        post_exit_hook,
        &profile,
    )
    .await?;
    Ok(mc_process)
}

/// Update playtime- sending a request to the server to update the playtime
#[tracing::instrument]
#[theseus_macros::debug_pin]
pub async fn try_update_playtime(path: &ProfilePathId) -> crate::Result<()> {
    let state = State::get().await?;

    let profile = get(path, None).await?.ok_or_else(|| {
        crate::ErrorKind::OtherError(format!(
            "Tried to update playtime for a nonexistent or unloaded profile at path {}!",
            path
        ))
    })?;
    let updated_recent_playtime = profile.metadata.recent_time_played;

    let res = if updated_recent_playtime > 0 {
        // Create update struct to send to Labrinth
        let modrinth_pack_version_id =
            profile.metadata.linked_data.and_then(|l| l.version_id);
        let playtime_update_json = json!({
            "seconds": updated_recent_playtime,
            "loader": profile.metadata.loader.to_string(),
            "game_version": profile.metadata.game_version,
            "parent": modrinth_pack_version_id,
        });
        // Copy this struct for every Modrinth project in the profile
        let mut hashmap: HashMap<String, serde_json::Value> = HashMap::new();
        for (_, project) in profile.projects {
            if let ProjectMetadata::Modrinth { version, .. } = project.metadata
            {
                hashmap.insert(version.id, playtime_update_json.clone());
            }
        }

        let creds = state.credentials.read().await;
        fetch::post_json(
            "https://api.modrinth.com/analytics/playtime",
            serde_json::to_value(hashmap)?,
            &state.fetch_semaphore,
            &creds,
        )
        .await
    } else {
        Ok(())
    };

    // If successful, update the profile metadata to match submitted
    if res.is_ok() {
        let mut profiles = state.profiles.write().await;
        if let Some(profile) = profiles.0.get_mut(path) {
            profile.metadata.submitted_time_played += updated_recent_playtime;
            profile.metadata.recent_time_played = 0;
        }
    }
    // Sync either way
    State::sync().await?;

    res
}

/// Creates a json configuration for a .mrpack zipped file
// Version ID of uploaded version (ie 1.1.5), not the unique identifying ID of the version (nvrqJg44)
#[tracing::instrument(skip_all)]
pub async fn create_mrpack_json(
    profile: &Profile,
    version_id: String,
    description: Option<String>,
) -> crate::Result<PackFormat> {
    // Add loader version to dependencies
    let mut dependencies = HashMap::new();
    match (
        profile.metadata.loader,
        profile.metadata.loader_version.clone(),
    ) {
        (crate::prelude::ModLoader::Forge, Some(v)) => {
            dependencies.insert(PackDependency::Forge, v.id)
        }
        (crate::prelude::ModLoader::NeoForge, Some(v)) => {
            dependencies.insert(PackDependency::NeoForge, v.id)
        }
        (crate::prelude::ModLoader::Fabric, Some(v)) => {
            dependencies.insert(PackDependency::FabricLoader, v.id)
        }
        (crate::prelude::ModLoader::Quilt, Some(v)) => {
            dependencies.insert(PackDependency::QuiltLoader, v.id)
        }
        (crate::prelude::ModLoader::Vanilla, _) => None,
        _ => {
            return Err(crate::ErrorKind::OtherError(
                "Loader version mismatch".to_string(),
            )
            .into())
        }
    };
    dependencies.insert(
        PackDependency::Minecraft,
        profile.metadata.game_version.clone(),
    );

    // Converts a HashMap<String, String> to a HashMap<String, String>
    // But the values are sanitized to only include the version number
    let dependencies = dependencies
        .into_iter()
        .map(|(k, v)| (k, sanitize_loader_version_string(&v, k).to_string()))
        .collect::<HashMap<_, _>>();

    let files: Result<Vec<PackFile>, crate::ErrorKind> = profile
        .projects
        .iter()
        .filter_map(|(mod_path, project)| {
            let path = mod_path.get_inner_path_unix();

            // Only Modrinth projects have a modrinth metadata field for the modrinth.json
            Some(Ok(match project.metadata {
                crate::prelude::ProjectMetadata::Modrinth {
                    ref version,
                    ..
                } => {
                    let mut env = HashMap::new();
                    // TODO: envtype should be a controllable option (in general or at least .mrpack exporting)
                    // For now, assume required.
                    // env.insert(EnvType::Client, project.client_side.clone());
                    // env.insert(EnvType::Server, project.server_side.clone());
                    env.insert(EnvType::Client, SideType::Required);
                    env.insert(EnvType::Server, SideType::Required);

                    let primary_file = if let Some(primary_file) =
                        version.files.first()
                    {
                        primary_file
                    } else {
                        return Some(Err(crate::ErrorKind::OtherError(
                            format!("No primary file found for mod at: {path}"),
                        )));
                    };

                    let file_size = primary_file.size;
                    let downloads = vec![primary_file.url.clone()];
                    let hashes = primary_file
                        .hashes
                        .clone()
                        .into_iter()
                        .map(|(h1, h2)| (PackFileHash::from(h1), h2))
                        .collect();

                    PackFile {
                        path,
                        hashes,
                        env: Some(env),
                        downloads,
                        file_size,
                    }
                }
                // Inferred files are skipped for the modrinth.json
                crate::prelude::ProjectMetadata::Inferred { .. } => {
                    return None
                }
                // Unknown projects are skipped for the modrinth.json
                crate::prelude::ProjectMetadata::Unknown => return None,
            }))
        })
        .collect();
    let files = files?;

    Ok(PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id,
        name: profile.metadata.name.clone(),
        summary: description,
        files,
        dependencies,
    })
}

fn sanitize_loader_version_string(s: &str, loader: PackDependency) -> &str {
    match loader {
        // Split on '-'
        // If two or more, take the second
        // If one, take the first
        // If none, take the whole thing
        PackDependency::Forge | PackDependency::NeoForge => {
            if s.starts_with("1.") {
                let mut split: std::str::Split<'_, char> = s.split('-');
                match split.next() {
                    Some(first) => match split.next() {
                        Some(second) => second,
                        None => first,
                    },
                    None => s,
                }
            } else {
                s
            }
        }
        // For quilt, etc we take the whole thing, as it functions like: 0.20.0-beta.11 (and should not be split here)
        PackDependency::QuiltLoader
        | PackDependency::FabricLoader
        | PackDependency::Minecraft => s,
    }
}

// Given a folder path, populate a Vec of all the files in the folder, recursively
#[async_recursion::async_recursion]
pub async fn add_all_recursive_folder_paths(
    path: &Path,
    path_list: &mut Vec<PathBuf>,
) -> crate::Result<()> {
    let mut read_dir = io::read_dir(path).await?;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, path))?
    {
        let path = entry.path();
        if path.is_dir() {
            add_all_recursive_folder_paths(&path, path_list).await?;
        } else {
            path_list.push(path);
        }
    }
    Ok(())
}

pub fn sanitize_profile_name(input: &str) -> String {
    input.replace(['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>'], "_")
}
