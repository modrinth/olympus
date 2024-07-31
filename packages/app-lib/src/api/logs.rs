use std::io::{Read, SeekFrom};
use std::time::SystemTime;

use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::{
    prelude::Credentials,
    util::io::{self, IOError},
    State,
};

#[derive(Serialize, Debug)]
pub struct Logs {
    pub log_type: LogType,
    pub filename: String,
    pub age: u64,
    pub output: Option<CensoredString>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum LogType {
    InfoLog,
    CrashReport,
}

#[derive(Serialize, Debug)]
pub struct LatestLogCursor {
    pub cursor: u64,
    pub output: CensoredString,
    pub new_file: bool,
}

#[derive(Serialize, Debug)] // Not deserialize
#[serde(transparent)]
pub struct CensoredString(String);
impl CensoredString {
    pub fn censor(mut s: String, credentials_set: &Vec<Credentials>) -> Self {
        let username = whoami::username();
        s = s
            .replace(&format!("/{}/", username), "/{COMPUTER_USERNAME}/")
            .replace(&format!("\\{}\\", username), "\\{COMPUTER_USERNAME}\\");
        for credentials in credentials_set {
            s = s
                .replace(&credentials.access_token, "{MINECRAFT_ACCESS_TOKEN}")
                .replace(&credentials.username, "{MINECRAFT_USERNAME}")
                .replace(
                    &credentials.id.as_simple().to_string(),
                    "{MINECRAFT_UUID}",
                )
                .replace(
                    &credentials.id.as_hyphenated().to_string(),
                    "{MINECRAFT_UUID}",
                );
        }

        Self(s)
    }
}

impl Logs {
    async fn build(
        log_type: LogType,
        age: SystemTime,
        profile_subpath: &str,
        filename: String,
        clear_contents: Option<bool>,
    ) -> crate::Result<Self> {
        Ok(Self {
            log_type,
            age: age
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
            output: if clear_contents.unwrap_or(false) {
                None
            } else {
                Some(
                    get_output_by_filename(
                        profile_subpath,
                        log_type,
                        &filename,
                    )
                    .await?,
                )
            },
            filename,
        })
    }
}

#[tracing::instrument]
pub async fn get_logs_from_type(
    profile_path: &str,
    log_type: LogType,
    clear_contents: Option<bool>,
    logs: &mut Vec<crate::Result<Logs>>,
) -> crate::Result<()> {
    let state = State::get().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.profile_logs_dir(profile_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(profile_path)
        }
    };

    if logs_folder.exists() {
        for entry in std::fs::read_dir(&logs_folder)
            .map_err(|e| IOError::with_path(e, &logs_folder))?
        {
            let entry: std::fs::DirEntry =
                entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
            let age = entry
                .metadata()?
                .created()
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_string();
                logs.push(
                    Logs::build(
                        log_type,
                        age,
                        profile_path,
                        file_name,
                        clear_contents,
                    )
                    .await,
                );
            }
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn get_logs(
    profile_path_id: &str,
    clear_contents: Option<bool>,
) -> crate::Result<Vec<Logs>> {
    let mut logs = Vec::new();
    get_logs_from_type(
        profile_path_id,
        LogType::InfoLog,
        clear_contents,
        &mut logs,
    )
    .await?;
    get_logs_from_type(
        profile_path_id,
        LogType::CrashReport,
        clear_contents,
        &mut logs,
    )
    .await?;

    let mut logs = logs.into_iter().collect::<crate::Result<Vec<Logs>>>()?;
    logs.sort_by(|a, b| b.age.cmp(&a.age).then(b.filename.cmp(&a.filename)));
    Ok(logs)
}

#[tracing::instrument]
pub async fn get_logs_by_filename(
    profile_path: &str,
    log_type: LogType,
    filename: String,
) -> crate::Result<Logs> {
    let state = State::get().await?;

    let path = match log_type {
        LogType::InfoLog => state.directories.profile_logs_dir(profile_path),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(profile_path)
        }
    }
    .join(&filename);

    let metadata = std::fs::metadata(&path)?;
    let age = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);

    Logs::build(log_type, age, profile_path, filename, Some(true)).await
}

#[tracing::instrument]
pub async fn get_output_by_filename(
    profile_subpath: &str,
    log_type: LogType,
    file_name: &str,
) -> crate::Result<CensoredString> {
    let state = State::get().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.profile_logs_dir(profile_subpath),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(profile_subpath)
        }
    };

    let path = logs_folder.join(file_name);

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect();

    // Load .gz file into String
    if let Some(ext) = path.extension() {
        if ext == "gz" {
            let file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            let mut contents = [0; 1024];
            let mut result = String::new();
            let mut gz =
                flate2::read::GzDecoder::new(std::io::BufReader::new(file));

            while gz
                .read(&mut contents)
                .map_err(|e| IOError::with_path(e, &path))?
                > 0
            {
                result.push_str(&String::from_utf8_lossy(&contents));
                contents = [0; 1024];
            }
            return Ok(CensoredString::censor(result, &credentials));
        } else if ext == "log" || ext == "txt" {
            let mut result = String::new();
            let mut contents = [0; 1024];
            let mut file = std::fs::File::open(&path)
                .map_err(|e| IOError::with_path(e, &path))?;
            // iteratively read the file to a String
            while file
                .read(&mut contents)
                .map_err(|e| IOError::with_path(e, &path))?
                > 0
            {
                result.push_str(&String::from_utf8_lossy(&contents));
                contents = [0; 1024];
            }
            let result = CensoredString::censor(result, &credentials);
            return Ok(result);
        }
    }
    Err(crate::ErrorKind::OtherError(format!(
        "File extension not supported: {}",
        path.display()
    ))
    .into())
}

#[tracing::instrument]
pub async fn delete_logs(profile_path_id: &str) -> crate::Result<()> {
    let state = State::get().await?;

    let logs_folder = state.directories.profile_logs_dir(profile_path_id);
    for entry in std::fs::read_dir(&logs_folder)
        .map_err(|e| IOError::with_path(e, &logs_folder))?
    {
        let entry = entry.map_err(|e| IOError::with_path(e, &logs_folder))?;
        let path = entry.path();
        if path.is_dir() {
            io::remove_dir_all(&path).await?;
        }
    }
    Ok(())
}

#[tracing::instrument]
pub async fn delete_logs_by_filename(
    profile_path_id: &str,
    log_type: LogType,
    filename: &str,
) -> crate::Result<()> {
    let state = State::get().await?;

    let logs_folder = match log_type {
        LogType::InfoLog => state.directories.profile_logs_dir(profile_path_id),
        LogType::CrashReport => {
            state.directories.crash_reports_dir(profile_path_id)
        }
    };

    let path = logs_folder.join(filename);
    io::remove_file(&path).await?;
    Ok(())
}

#[tracing::instrument]
pub async fn get_latest_log_cursor(
    profile_path: &str,
    cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    get_generic_live_log_cursor(profile_path, "latest.log", cursor).await
}

#[tracing::instrument]
pub async fn get_generic_live_log_cursor(
    profile_path_id: &str,
    log_file_name: &str,
    mut cursor: u64, // 0 to start at beginning of file
) -> crate::Result<LatestLogCursor> {
    let state = State::get().await?;
    let logs_folder = state.directories.profile_logs_dir(profile_path_id);
    let path = logs_folder.join(log_file_name);
    if !path.exists() {
        // Allow silent failure if latest.log doesn't exist (as the instance may have been launched, but not yet created the file)
        return Ok(LatestLogCursor {
            cursor: 0,
            new_file: false,
            output: CensoredString("".to_string()),
        });
    }

    let mut file = File::open(&path)
        .await
        .map_err(|e| IOError::with_path(e, &path))?;
    let metadata = file
        .metadata()
        .await
        .map_err(|e| IOError::with_path(e, &path))?;

    let mut new_file = false;
    if cursor > metadata.len() {
        // Cursor is greater than file length, reset cursor to 0
        // Likely cause is that the file was rotated while the log was being read
        cursor = 0;
        new_file = true;
    }

    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(cursor))
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Seek to cursor
    let bytes_read = file
        .read_to_end(&mut buffer)
        .map_err(|e| IOError::with_path(e, &path))
        .await?; // Read to end of file
    let output = String::from_utf8_lossy(&buffer).to_string(); // Convert to String
    let cursor = cursor + bytes_read as u64; // Update cursor

    let credentials = Credentials::get_all(&state.pool)
        .await?
        .into_iter()
        .map(|x| x.1)
        .collect();
    let output = CensoredString::censor(output, &credentials);
    Ok(LatestLogCursor {
        cursor,
        new_file,
        output,
    })
}
