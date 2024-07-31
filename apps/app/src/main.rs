#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use theseus::prelude::*;

mod api;
mod error;

#[cfg(target_os = "macos")]
mod macos;

// Should be called in launcher initialization
#[tracing::instrument(skip_all)]
#[tauri::command]
async fn initialize_state(app: tauri::AppHandle) -> api::Result<()> {
    theseus::EventState::init(app).await?;
    State::init().await?;

    Ok(())
}

#[tauri::command]
fn is_dev() -> bool {
    cfg!(debug_assertions)
}

/// Show Main Window - called once after first frontend rendered
#[tauri::command]
fn show_window(window: tauri::Window) {
    window.get_window("main").unwrap().show().unwrap();
}

// Toggles decorations
#[tauri::command]
async fn toggle_decorations(b: bool, window: tauri::Window) -> api::Result<()> {
    window.set_decorations(b).map_err(|e| {
        theseus::Error::from(theseus::ErrorKind::OtherError(format!(
            "Failed to toggle decorations: {}",
            e
        )))
    })?;
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

// if Tauri app is called with arguments, then those arguments will be treated as commands
// ie: deep links or filepaths for .mrpacks
fn main() {
    tauri_plugin_deep_link::prepare("ModrinthApp");

    /*
        tracing is set basd on the environment variable RUST_LOG=xxx, depending on the amount of logs to show
            ERROR > WARN > INFO > DEBUG > TRACE
        eg. RUST_LOG=info will show info, warn, and error logs
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)
            RUST_LOG="theseus=trace" will show *all* messages but from theseus only (and not dependencies using similar crates)

        Error messages returned to Tauri will display as traced error logs if they return an error.
        This will also include an attached span trace if the error is from a tracing error, and the level is set to info, debug, or trace

        on unix:
            RUST_LOG="theseus=trace" {run command}

    */
    let _log_guard = theseus::start_logger();

    tracing::info!("Initialized tracing subscriber. Loading Modrinth App!");

    let mut builder = tauri::Builder::default();
    builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Register deep link handler, allowing reading of modrinth:// links
            if let Err(e) = tauri_plugin_deep_link::register(
                "modrinth",
                |request: String| {
                    tauri::async_runtime::spawn(api::utils::handle_command(
                        request,
                    ));
                },
            ) {
                // Allow it to fail- see https://github.com/FabianLars/tauri-plugin-deep-link/issues/19
                tracing::error!("Error registering deep link handler: {}", e);
            }

            let win = app.get_window("main").unwrap();
            win.hide().unwrap();

            #[cfg(not(target_os = "linux"))]
            {
                use window_shadows::set_shadow;
                set_shadow(&win, true).unwrap();
            }
            #[cfg(target_os = "macos")]
            {
                use macos::window_ext::WindowExt;
                win.set_transparent_titlebar(true);
                win.position_traffic_lights(9.0, 16.0);

                macos::delegate::register_open_file(|filename| {
                    tauri::async_runtime::spawn(api::utils::handle_command(
                        filename,
                    ));
                })
                .unwrap();
            }

            Ok(())
        });

    #[cfg(target_os = "macos")]
    {
        use tauri::WindowEvent;
        builder = builder.on_window_event(|e| {
            use macos::window_ext::WindowExt;
            if let WindowEvent::Resized(..) = e.event() {
                let win = e.window();
                win.position_traffic_lights(9.0, 16.0);
            }
        })
    }

    let builder = builder
        .plugin(api::auth::init())
        .plugin(api::mr_auth::init())
        .plugin(api::import::init())
        .plugin(api::logs::init())
        .plugin(api::jre::init())
        .plugin(api::metadata::init())
        .plugin(api::pack::init())
        .plugin(api::process::init())
        .plugin(api::profile::init())
        .plugin(api::profile_create::init())
        .plugin(api::settings::init())
        .plugin(api::tags::init())
        .plugin(api::utils::init())
        .plugin(api::cache::init())
        .invoke_handler(tauri::generate_handler![
            initialize_state,
            is_dev,
            toggle_decorations,
            show_window,
            api::auth::auth_login,
            api::mr_auth::modrinth_auth_login,
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
