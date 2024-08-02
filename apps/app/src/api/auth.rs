use crate::api::Result;
use chrono::{Duration, Utc};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, UserAttentionType};
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("auth")
        .invoke_handler(tauri::generate_handler![
            auth_get_default_user,
            auth_set_default_user,
            auth_remove_user,
            auth_users,
            auth_refresh,
        ])
        .build()
}

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously, returning a URL to visit (that the user will sign in at)
#[tauri::command]
pub async fn auth_login(app: tauri::AppHandle) -> Result<Option<Credentials>> {
    let flow = minecraft_auth::begin_login().await?;

    let start = Utc::now();

    if let Some(window) = app.get_window("signin") {
        window.close()?;
    }

    let window = tauri::WindowBuilder::new(
        &app,
        "signin",
        tauri::WindowUrl::External(flow.redirect_uri.parse().map_err(
            |_| {
                theseus::ErrorKind::OtherError(
                    "Error parsing auth redirect URL".to_string(),
                )
                .as_error()
            },
        )?),
    )
    .title("Sign into Modrinth")
    .always_on_top(true)
    .center()
    .build()?;

    window.request_user_attention(Some(UserAttentionType::Critical))?;

    while (Utc::now() - start) < Duration::minutes(10) {
        if window.title().is_err() {
            // user closed window, cancelling flow
            return Ok(None);
        }

        if window
            .url()
            .as_str()
            .starts_with("https://login.live.com/oauth20_desktop.srf")
        {
            if let Some((_, code)) =
                window.url().query_pairs().find(|x| x.0 == "code")
            {
                window.close()?;
                let val =
                    minecraft_auth::finish_login(&code.clone(), flow).await?;

                return Ok(Some(val));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    window.close()?;
    Ok(None)
}
#[tauri::command]
pub async fn auth_remove_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::remove_user(user).await?)
}

#[tauri::command]
pub async fn auth_get_default_user() -> Result<Option<uuid::Uuid>> {
    Ok(minecraft_auth::get_default_user().await?)
}

#[tauri::command]
pub async fn auth_set_default_user(user: uuid::Uuid) -> Result<()> {
    Ok(minecraft_auth::set_default_user(user).await?)
}

/// Get a copy of the list of all user credentials
// invoke('plugin:auth|auth_users',user)
#[tauri::command]
pub async fn auth_users() -> Result<Vec<Credentials>> {
    Ok(minecraft_auth::users().await?)
}

/// Refreshes a credential
// invoke('plugin:auth|auth_refresh',user)
#[tauri::command]
pub async fn auth_refresh(user: uuid::Uuid) -> Result<Credentials> {
    Ok(minecraft_auth::refresh(user).await?)
}
