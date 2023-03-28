use crate::api::{Result, TheseusGuiError::ProfileCreation};
use daedalus::modded::LoaderVersion;
use futures::prelude::*;
use std::path::PathBuf;
use theseus::prelude::*;
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;

const DEFAULT_NAME: &'static str = "Untitled Instance";
const PROFILE_FILE_PATH: &'static str = "../.minecraft";

// Generic basic profile creation tool.
// Creates an essentially empty dummy profile with profile_create
#[tauri::command]
pub async fn profile_create_empty() -> Result<Profile> {
    Ok(profile_create(
        PathBuf::from(PROFILE_FILE_PATH), // the path of the newly created profile
        String::from(DEFAULT_NAME),       // the name of the profile
        String::from("1.8"),              // the game version of the profile
        None,                             // the icon for the profile
        ModLoader::Vanilla,               // the modloader to use
        String::from("stable"), // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
    )
    .await?)
}

// Creates a profile at  the given filepath and adds it to the in-memory state
// This is reused mostly from the CLI. TODO: touch up.
// invoke('profile_add',profile)
#[tauri::command]
pub async fn profile_create(
    path: PathBuf,          // the path of the newly created profile
    name: String,           // the name of the profile
    game_version: String,   // the game version of the profile
    icon: Option<PathBuf>,  // the icon for the profile
    modloader: ModLoader,   // the modloader to use
    loader_version: String, // the modloader version to use, set to "latest", "stable", or the ID of your chosen loader
) -> Result<Profile> {
    // TODO: validate inputs from args early
    let state = State::get().await?;

    if path.exists() {
        if path.is_dir() {
            return Err(ProfileCreation(
                "Attempted to create profile in something other than a folder!"
                    .to_string(),
            ));
        }
        if !path.join("profile.json").exists() {}

        if ReadDirStream::new(fs::read_dir(&path).await?)
            .next()
            .await
            .is_some()
        {
            // TODO: in CLI, we have manual override for this
            return Err(ProfileCreation(
                "You are trying to create a profile in a non-empty directory!"
                    .to_string(),
            ));
        }
    } else {
        fs::create_dir_all(&path).await?;
    }
    println!(
        "Creating profile at path {}",
        &path.canonicalize()?.display()
    );

    let loader = modloader;
    let loader = if loader != ModLoader::Vanilla {
        let version = loader_version;

        let filter = |it: &LoaderVersion| match version.as_str() {
            "latest" => true,
            "stable" => it.stable,
            id => it.id == String::from(id),
        };

        let loader_data = match loader {
            ModLoader::Forge => &state.metadata.forge,
            ModLoader::Fabric => &state.metadata.fabric,
            _ => return Err(ProfileCreation(format!("Could not get manifest for loader {loader}. This is a bug in the GUI!")))
        };

        let ref loaders = loader_data.game_versions
            .iter()
            .find(|it| it.id == game_version)
            .ok_or_else(|| ProfileCreation(format!("Modloader {loader} unsupported for Minecraft version {game_version}!")))?
            .loaders;

        let loader_version =
            loaders.iter().cloned().find(filter).ok_or_else(|| {
                ProfileCreation(format!(
                    "Invalid version {version} for modloader {loader}"
                ))
            })?;

        Some((loader_version, loader))
    } else {
        None
    };

    let mut profile = Profile::new(name, game_version, path.clone()).await?;

    if let Some(ref icon) = icon {
        profile.with_icon(icon).await?;
    }

    if let Some((loader_version, loader)) = loader {
        profile.with_loader(loader, Some(loader_version));
    }

    profile::add(profile.clone()).await?;
    State::sync().await?;

    Ok(profile)
}