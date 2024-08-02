use std::{
    fmt,
    path::{Path, PathBuf},
};

use io::IOError;
use serde::{Deserialize, Serialize};

use crate::{
    event::{
        emit::{emit_loading, init_or_edit_loading},
        LoadingBarId,
    },
    util::{
        fetch::{self, IoSemaphore},
        io,
    },
};

pub mod atlauncher;
pub mod curseforge;
pub mod gdlauncher;
pub mod mmc;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportLauncherType {
    MultiMC,
    PrismLauncher,
    ATLauncher,
    GDLauncher,
    Curseforge,
    Mojang,
    #[serde(other)]
    Unknown,
}
// impl display
impl fmt::Display for ImportLauncherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportLauncherType::MultiMC => write!(f, "MultiMC"),
            ImportLauncherType::PrismLauncher => write!(f, "PrismLauncher"),
            ImportLauncherType::ATLauncher => write!(f, "ATLauncher"),
            ImportLauncherType::GDLauncher => write!(f, "GDLauncher"),
            ImportLauncherType::Curseforge => write!(f, "Curseforge"),
            ImportLauncherType::Mojang => write!(f, "Mojang"),
            ImportLauncherType::Unknown => write!(f, "Unknown"),
        }
    }
}

// Return a list of importable instances from a launcher type and base path, by iterating through the folder and checking
pub async fn get_importable_instances(
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
) -> crate::Result<Vec<String>> {
    // Some launchers have a different folder structure for instances
    let instances_subfolder = match launcher_type {
        ImportLauncherType::GDLauncher | ImportLauncherType::ATLauncher => {
            "instances".to_string()
        }
        ImportLauncherType::Curseforge => "Instances".to_string(),
        ImportLauncherType::MultiMC => {
            mmc::get_instances_subpath(base_path.clone().join("multimc.cfg"))
                .await
                .unwrap_or_else(|| "instances".to_string())
        }
        ImportLauncherType::PrismLauncher => mmc::get_instances_subpath(
            base_path.clone().join("prismlauncher.cfg"),
        )
        .await
        .unwrap_or_else(|| "instances".to_string()),
        ImportLauncherType::Mojang => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unsupported".to_string(),
            )
            .into())
        },
        ImportLauncherType::Unknown => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unknown".to_string(),
            )
            .into())
        }
    };

    let instances_folder = base_path.join(&instances_subfolder);
    let mut instances = Vec::new();
    let mut dir = io::read_dir(&instances_folder).await.map_err(| _ | {
        crate::ErrorKind::InputError(format!(
            "Invalid {launcher_type} launcher path, could not find '{instances_subfolder}' subfolder."
        ))
    })?;
    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, &instances_folder))?
    {
        let path = entry.path();
        if path.is_dir() {
            // Check instance is valid of this launcher type
            if is_valid_importable_instance(path.clone(), launcher_type).await {
                let name = path.file_name();
                if let Some(name) = name {
                    instances.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(instances)
}

// Import an instance from a launcher type and base path
// Note: this *deletes* the submitted empty profile

#[tracing::instrument]
pub async fn import_instance(
    profile_path: &str, // This should be a blank profile
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
    instance_folder: String,
) -> crate::Result<()> {
    tracing::debug!("Importing instance from {instance_folder}");
    let res = match launcher_type {
        ImportLauncherType::MultiMC | ImportLauncherType::PrismLauncher => {
            mmc::import_mmc(
                base_path,       // path to base mmc folder
                instance_folder, // instance folder in mmc_base_path
                profile_path,    // path to profile
            )
            .await
        }
        ImportLauncherType::ATLauncher => {
            atlauncher::import_atlauncher(
                base_path,       // path to atlauncher folder
                instance_folder, // instance folder in atlauncher
                profile_path,    // path to profile
            )
            .await
        }
        ImportLauncherType::GDLauncher => {
            gdlauncher::import_gdlauncher(
                base_path.join("instances").join(instance_folder), // path to gdlauncher folder
                profile_path, // path to profile
            )
            .await
        }
        ImportLauncherType::Curseforge => {
            curseforge::import_curseforge(
                base_path.join("Instances").join(instance_folder), // path to curseforge folder
                profile_path, // path to profile
            )
            .await
        }
        ImportLauncherType::Mojang => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unsupported".to_string(),
            )
            .into())
        },
        ImportLauncherType::Unknown => {
            return Err(crate::ErrorKind::InputError(
                "Launcher type Unknown".to_string(),
            )
            .into());
        }
    };

    // If import failed, delete the profile
    match res {
        Ok(_) => {}
        Err(e) => {
            tracing::warn!("Import failed: {:?}", e);
            let _ = crate::api::profile::remove(profile_path).await;
            return Err(e);
        }
    }

    tracing::debug!("Completed import.");
    Ok(())
}

/// Returns the default path for the given launcher type
/// None if it can't be found or doesn't exist
pub fn get_default_launcher_path(
    r#type: ImportLauncherType,
) -> Option<PathBuf> {
    let path = match r#type {
        ImportLauncherType::MultiMC => None, // multimc data is *in* app dir
        ImportLauncherType::PrismLauncher => {
            Some(dirs::data_dir()?.join("PrismLauncher"))
        }
        ImportLauncherType::ATLauncher => {
            Some(dirs::data_dir()?.join("ATLauncher"))
        }
        ImportLauncherType::GDLauncher => {
            Some(dirs::data_dir()?.join("gdlauncher_next"))
        }
        ImportLauncherType::Curseforge => {
            Some(dirs::home_dir()?.join("curseforge").join("minecraft"))
        }
        ImportLauncherType::Mojang => {
            Some(dirs::data_dir()?.join(".minecraft"))
        }
        ImportLauncherType::Unknown => None,
    };
    let path = path?;
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Checks if this PathBuf is a valid instance for the given launcher type

#[tracing::instrument]
pub async fn is_valid_importable_instance(
    instance_path: PathBuf,
    r#type: ImportLauncherType,
) -> bool {
    match r#type {
        ImportLauncherType::MultiMC | ImportLauncherType::PrismLauncher => {
            mmc::is_valid_mmc(instance_path).await
        }
        ImportLauncherType::ATLauncher => {
            atlauncher::is_valid_atlauncher(instance_path).await
        }
        ImportLauncherType::GDLauncher => {
            gdlauncher::is_valid_gdlauncher(instance_path).await
        }
        ImportLauncherType::Curseforge => {
            curseforge::is_valid_curseforge(instance_path).await
        }
        ImportLauncherType::Mojang => false,
        ImportLauncherType::Unknown => false,
    }
}

/// Caches an image file in the filesystem into the cache directory, and returns the path to the cached file.

#[tracing::instrument]
pub async fn recache_icon(
    icon_path: PathBuf,
) -> crate::Result<Option<PathBuf>> {
    let state = crate::State::get().await?;

    let bytes = tokio::fs::read(&icon_path).await;
    if let Ok(bytes) = bytes {
        let bytes = bytes::Bytes::from(bytes);
        let cache_dir = &state.directories.caches_dir();
        let semaphore = &state.io_semaphore;
        Ok(Some(
            fetch::write_cached_icon(
                &icon_path.to_string_lossy(),
                cache_dir,
                bytes,
                semaphore,
            )
            .await?,
        ))
    } else {
        // could not find icon (for instance, prism default icon, etc)
        Ok(None)
    }
}

pub async fn copy_dotminecraft(
    profile_path_id: &str,
    dotminecraft: PathBuf,
    io_semaphore: &IoSemaphore,
    existing_loading_bar: Option<LoadingBarId>,
) -> crate::Result<LoadingBarId> {
    // Get full path to profile
    let profile_path =
        crate::api::profile::get_full_path(profile_path_id).await?;

    // Gets all subfiles recursively in src
    let subfiles = get_all_subfiles(&dotminecraft).await?;
    let total_subfiles = subfiles.len() as u64;

    let loading_bar = init_or_edit_loading(
        existing_loading_bar,
        crate::LoadingBarType::CopyProfile {
            import_location: dotminecraft.clone(),
            profile_name: profile_path_id.to_string(),
        },
        total_subfiles as f64,
        "Copying files in profile",
    )
    .await?;

    // Copy each file
    for src_child in subfiles {
        let dst_child =
            src_child.strip_prefix(&dotminecraft).map_err(|_| {
                crate::ErrorKind::InputError(format!(
                    "Invalid file: {}",
                    &src_child.display()
                ))
            })?;
        let dst_child = profile_path.join(dst_child);

        // sleep for cpu for 1 millisecond
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;

        fetch::copy(&src_child, &dst_child, io_semaphore).await?;

        emit_loading(&loading_bar, 1.0, None).await?;
    }
    Ok(loading_bar)
}

/// Recursively get a list of all subfiles in src
/// uses async recursion

#[async_recursion::async_recursion]
#[tracing::instrument]
pub async fn get_all_subfiles(src: &Path) -> crate::Result<Vec<PathBuf>> {
    if !src.is_dir() {
        return Ok(vec![src.to_path_buf()]);
    }

    let mut files = Vec::new();
    let mut dir = io::read_dir(&src).await?;
    while let Some(child) = dir
        .next_entry()
        .await
        .map_err(|e| IOError::with_path(e, src))?
    {
        let src_child = child.path();
        files.append(&mut get_all_subfiles(&src_child).await?);
    }
    Ok(files)
}
