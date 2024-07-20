//! Theseus profile management interface
use crate::pack::install_from::CreatePackProfile;
use crate::prelude::ProfilePathId;
use crate::state::LinkedData;
use crate::util::io::{self, canonicalize};
use crate::{
    event::{emit::emit_profile, ProfilePayloadType},
    prelude::ModLoader,
};
use crate::{pack, profile, ErrorKind};
pub use crate::{
    state::{JavaSettings, Profile},
    State,
};
use daedalus::modded::LoaderVersion;
use std::path::PathBuf;

use tracing::{info, trace};
use uuid::Uuid;

// Creates a profile of a given name and adds it to the in-memory state
// Returns relative filepath as ProfilePathId which can be used to access it in the State
#[tracing::instrument]
#[theseus_macros::debug_pin]
#[allow(clippy::too_many_arguments)]
pub async fn profile_create(
    mut name: String, // the name of the profile, and relative path
    game_version: String, // the game version of the profile
    modloader: ModLoader, // the modloader to use
    loader_version: Option<String>, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader. defaults to latest
    icon: Option<PathBuf>,          // the icon for the profile
    icon_url: Option<String>, // the URL icon for a profile (ONLY USED FOR TEMPORARY PROFILES)
    linked_data: Option<LinkedData>, // the linked project ID (mainly for modpacks)- used for updating
    skip_install_profile: Option<bool>,
    no_watch: Option<bool>,
) -> crate::Result<ProfilePathId> {
    name = profile::sanitize_profile_name(&name);

    trace!("Creating new profile. {}", name);
    let state = State::get().await?;
    let uuid = Uuid::new_v4();

    let mut path = state.directories.profiles_dir().await.join(&name);

    if path.exists() {
        let mut new_name;
        let mut new_path;
        let mut which = 1;
        loop {
            new_name = format!("{name} ({which})");
            new_path = state.directories.profiles_dir().await.join(&new_name);
            if !new_path.exists() {
                break;
            }
            which += 1;
        }

        tracing::debug!(
            "Folder collision: {}, renaming to: {}",
            path.display(),
            new_path.display()
        );
        path = new_path;
        name = new_name;
    }
    io::create_dir_all(&path).await?;

    info!(
        "Creating profile at path {}",
        &canonicalize(&path)?.display()
    );
    let loader = if modloader != ModLoader::Vanilla {
        get_loader_version_from_loader(
            game_version.clone(),
            modloader,
            loader_version,
        )
        .await?
    } else {
        None
    };

    let mut profile = Profile::new(uuid, name, game_version).await?;
    let result = async {
        if let Some(ref icon) = icon {
            let bytes =
                io::read(state.directories.caches_dir().join(icon)).await?;
            profile
                .set_icon_file(
                    &state.directories.caches_dir(),
                    &state.io_semaphore,
                    bytes::Bytes::from(bytes),
                    &icon.to_string_lossy(),
                )
                .await?;
        }

        profile.metadata.icon_url = icon_url;
        if let Some(loader_version) = loader {
            profile.metadata.loader = modloader;
            profile.metadata.loader_version = Some(loader_version);
        }

        profile.metadata.linked_data = linked_data;
        if let Some(linked_data) = &mut profile.metadata.linked_data {
            linked_data.locked = Some(
                linked_data.project_id.is_some()
                    && linked_data.version_id.is_some(),
            );
        }

        emit_profile(
            uuid,
            &profile.profile_id(),
            &profile.metadata.name,
            ProfilePayloadType::Created,
        )
        .await?;

        {
            let mut profiles = state.profiles.write().await;
            profiles
                .insert(profile.clone(), no_watch.unwrap_or_default())
                .await?;
        }

        if !skip_install_profile.unwrap_or(false) {
            crate::launcher::install_minecraft(&profile, None, false).await?;
        }
        State::sync().await?;

        Ok(profile.profile_id())
    }
    .await;

    match result {
        Ok(profile) => Ok(profile),
        Err(err) => {
            let _ = crate::api::profile::remove(&profile.profile_id()).await;

            Err(err)
        }
    }
}

pub async fn profile_create_from_creator(
    profile: CreatePackProfile,
) -> crate::Result<ProfilePathId> {
    profile_create(
        profile.name,
        profile.game_version,
        profile.modloader,
        profile.loader_version,
        profile.icon,
        profile.icon_url,
        profile.linked_data,
        profile.skip_install_profile,
        profile.no_watch,
    )
    .await
}

pub async fn profile_create_from_duplicate(
    copy_from: ProfilePathId,
) -> crate::Result<ProfilePathId> {
    // Original profile
    let profile = profile::get(&copy_from, None).await?.ok_or_else(|| {
        ErrorKind::UnmanagedProfileError(copy_from.to_string())
    })?;

    let profile_path_id = profile_create(
        profile.metadata.name.clone(),
        profile.metadata.game_version.clone(),
        profile.metadata.loader,
        profile.metadata.loader_version.clone().map(|it| it.id),
        profile.metadata.icon.clone(),
        profile.metadata.icon_url.clone(),
        profile.metadata.linked_data.clone(),
        Some(true),
        Some(true),
    )
    .await?;

    // Copy it over using the import system (essentially importing from the same profile)
    let state = State::get().await?;
    let bar = pack::import::copy_dotminecraft(
        profile_path_id.clone(),
        copy_from.get_full_path().await?,
        &state.io_semaphore,
        None,
    )
    .await?;

    let duplicated_profile =
        profile::get(&profile_path_id, None).await?.ok_or_else(|| {
            ErrorKind::UnmanagedProfileError(profile_path_id.to_string())
        })?;

    crate::launcher::install_minecraft(&duplicated_profile, Some(bar), false)
        .await?;
    {
        let state = State::get().await?;
        let mut file_watcher = state.file_watcher.write().await;
        Profile::watch_fs(
            &profile.get_profile_full_path().await?,
            &mut file_watcher,
        )
        .await?;
    }

    // emit profile edited
    emit_profile(
        profile.uuid,
        &profile.profile_id(),
        &profile.metadata.name,
        ProfilePayloadType::Edited,
    )
    .await?;
    State::sync().await?;
    Ok(profile_path_id)
}

#[tracing::instrument]
#[theseus_macros::debug_pin]
pub(crate) async fn get_loader_version_from_loader(
    game_version: String,
    loader: ModLoader,
    loader_version: Option<String>,
) -> crate::Result<Option<LoaderVersion>> {
    let state = State::get().await?;
    let metadata = state.metadata.read().await;

    let version = loader_version.unwrap_or_else(|| "latest".to_string());

    let filter = |it: &LoaderVersion| match version.as_str() {
        "latest" => true,
        "stable" => it.stable,
        id => {
            it.id == *id
                || format!("{}-{}", game_version, id) == it.id
                || format!("{}-{}-{}", game_version, id, game_version) == it.id
        }
    };

    let loader_data = match loader {
        ModLoader::Forge => &metadata.forge,
        ModLoader::Fabric => &metadata.fabric,
        ModLoader::Quilt => &metadata.quilt,
        ModLoader::NeoForge => &metadata.neoforge,
        _ => {
            return Err(
                ProfileCreationError::NoManifest(loader.to_string()).into()
            )
        }
    };

    let loaders = &loader_data
        .game_versions
        .iter()
        .find(|it| {
            it.id
                .replace(daedalus::modded::DUMMY_REPLACE_STRING, &game_version)
                == game_version
        })
        .ok_or_else(|| {
            ProfileCreationError::ModloaderUnsupported(
                loader.to_string(),
                game_version.clone(),
            )
        })?
        .loaders;

    let loader_version = loaders
        .iter()
        .find(|&x| filter(x))
        .cloned()
        .or(
            // If stable was searched for but not found, return latest by default
            if version == "stable" {
                loaders.iter().next().cloned()
            } else {
                None
            },
        )
        .ok_or_else(|| {
            ProfileCreationError::InvalidVersionModloader(
                version,
                loader.to_string(),
            )
        })?;

    Ok(Some(loader_version))
}

#[derive(thiserror::Error, Debug)]
pub enum ProfileCreationError {
    #[error("Profile .json exists: {0}")]
    ProfileExistsError(PathBuf),
    #[error("Modloader {0} unsupported for Minecraft version {1}")]
    ModloaderUnsupported(String, String),
    #[error("Invalid version {0} for modloader {1}")]
    InvalidVersionModloader(String, String),
    #[error("Could not get manifest for loader {0}. This is a bug in the GUI")]
    NoManifest(String),
    #[error("Could not get State.")]
    NoState,

    #[error("Attempted to create project in something other than a folder.")]
    NotFolder,
    #[error("You are trying to create a profile in a non-empty directory")]
    NotEmptyFolder,

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
