use std::{collections::HashMap, path::PathBuf};

use crate::api::Result;
use theseus::{skin_manager::{self, SkinSave, SkinCache, Filters}, prelude::Credentials};

use uuid::Uuid;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("skin")
        .invoke_handler(tauri::generate_handler![
            skin_get_user_skin_data,
            skin_get_heads,
            skin_set_skin,
            skin_delete_skin,
            skin_import_skin,
            skin_check_image,
            skin_check_skin,
            skin_get_cape_data,
            skin_cache_users_skins,
            skin_cache_new_user_skin,
            skin_get_launcher_names,
            skin_save_skin,
            skin_get_skins,
            skin_set_cape,
            skin_get_order,
            skin_save_order,
            skin_get_filters,
            skin_save_filters
        ])
        .build()
}

// Get image size
// invoke('plugin:skin|check_image',path)
#[tauri::command]
pub async fn skin_check_image(path: String) -> Result<bool> {
    Ok(skin_manager::check_image(path).await?)
}

// Returns true if skin is not in library
// invoke('plugin:skin|check_skin',skin)
#[tauri::command]
pub async fn skin_check_skin(skin: String, id: Uuid) -> Result<bool> {
    Ok(skin_manager::check_skin(skin, id).await?)
}

// Returns filter options
// invoke('plugin:skin|get_filters')
#[tauri::command]
pub async fn skin_get_filters() -> Result<Filters> {
    Ok(skin_manager::get_filters().await?)
}

// Saves filter options
// invoke('plugin:skin|save_filters',sort)
#[tauri::command]
pub async fn skin_save_filters(filters: Filters) -> Result<bool> {
    Ok(skin_manager::save_filters(filters).await?)
}

// Sets player's skin
// invoke('plugin:skin|skin_set_skin', { skin, arms, user })
#[tauri::command]
pub async fn skin_set_skin(skin: String, arms: String, creds: Credentials) -> Result<bool> {
    Ok(skin_manager::set_skin(skin, arms, creds).await?)
}

// Sets the players cape
// invoke('plugin:skin|skin_set_cape', { capeid, token })
#[tauri::command]
pub async fn skin_set_cape(capeid: String, token: String) -> Result<bool> {
    Ok(skin_manager::set_cape(capeid, token).await?)
}

// Returns cape id or url
// invoke('plugin:skin|skin_set_cape', { cape, key })
#[tauri::command]
pub async fn skin_get_cape_data(cape: String, key: String) -> Result<String> {
    Ok(skin_manager::get_cape_data(cape, key).await?)
}

// Gets the current account's skin data
// invoke('plugin:skin|skin_get_user_data', { id })
#[tauri::command]
pub async fn skin_get_user_skin_data(id: Uuid) -> Result<SkinCache> {
    Ok(skin_manager::get_user_skin_data(id).await?)
}

// Makes api request to mojang, updating all accounts' skin data
// invoke('plugin:skin|skin_cache_users_skins')
#[tauri::command]
pub async fn skin_cache_users_skins() -> Result<bool> {
    Ok(skin_manager::cache_users_skins().await?)
}

// Makes api request to mojang, updating current account's skin data
// invoke('plugin:skin|skin_cache_new_user_skin', { user })
#[tauri::command]
pub async fn skin_cache_new_user_skin(creds: Credentials) -> Result<()> {
    Ok(skin_manager::cache_new_user_skin(creds).await?)
}

// Saves the skin data to the manager
// invoke('plugin:skin|skin_save_skin', { user, data, name, model, skinid })
#[tauri::command]
pub async fn skin_save_skin(user: Uuid, data: SkinCache, name: String, model: String, skinid: String) -> Result<()> {
    Ok(skin_manager::save_skin(user, data, name, model, skinid).await?)
}

// Updates skin saves from the manager
// invoke('plugin:skin|skin_update_skins', { id })
#[tauri::command]
pub async fn skin_delete_skin(id: Uuid) -> Result<()> {
    Ok(skin_manager::delete_skin(id).await?)
}

// Gets all account heads from the cache
// invoke('plugin:skin|skin_get_heads')
#[tauri::command]
pub async fn skin_get_heads() -> Result<HashMap<Uuid, String>> {
    Ok(skin_manager::get_heads().await?)
}

// Gets all saved skins from the manager
// invoke('plugin:skin|skin_get_skins')
#[tauri::command]
pub async fn skin_get_skins() -> Result<Vec<SkinSave>> {
    Ok(skin_manager::get_skins().await?)
}

// Gets custom skin order
// invoke('plugin:skin|skin_get_order')
#[tauri::command]
pub async fn skin_get_order(user: Uuid) -> Result<Vec<Uuid>> {
    Ok(skin_manager::get_order(user).await?)
}

// Saves custom skin order
// invoke('plugin:skin|skin_save_order')
#[tauri::command]
pub async fn skin_save_order(order: Vec<Uuid>, user: Uuid) -> Result<()> {
    Ok(skin_manager::save_order(order, user).await?)
}

// Gets a list of all saved skins in the mojang launcher
// invoke('plugin:skin|skin_get_launcher_names', { path })
#[tauri::command]
pub async fn skin_get_launcher_names(path: PathBuf, installer: String) -> Result<Vec<skin_manager::MojangNames>> {
    Ok(skin_manager::get_launcher_names(path, installer).await?)
}

// Adds the skin from the mojang launcher to the manager
// invoke('plugin:skin|skin_import_skin', { name, path, user })
#[tauri::command]
pub async fn skin_import_skin(id: String, path: PathBuf, installer: String) -> Result<SkinCache> {
    Ok(skin_manager::import_skin(id, path, installer).await?)
}