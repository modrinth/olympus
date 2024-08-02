use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use thiserror::Error;

pub mod auth;
pub mod import;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod mr_auth;
pub mod pack;
pub mod process;
pub mod profile;
pub mod profile_create;
pub mod settings;
pub mod tags;
pub mod utils;
pub mod skin_manager;

pub mod cache;

pub type Result<T> = std::result::Result<T, TheseusSerializableError>;

// // Main returnable Theseus GUI error
// // Needs to be Serializable to be returned to the JavaScript side
// #[derive(Error, Debug, Serialize)]
// pub enum TheseusGuiError {
//     #[error(transparent)]
//     Serializable(),
// }

// Serializable error intermediary, so TheseusGuiError can be Serializable (eg: so that we can return theseus::Errors in Tauri directly)
#[derive(Error, Debug)]
pub enum TheseusSerializableError {
    #[error("{0}")]
    Theseus(#[from] theseus::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[cfg(target_os = "macos")]
    #[error("Callback error: {0}")]
    Callback(String),
}

// Generic implementation of From<T> for ErrorTypeA
// impl<T> From<T> for TheseusGuiError
// where
//     TheseusSerializableError: From<T>,
// {
//     fn from(error: T) -> Self {
//         TheseusGuiError::Serializable(TheseusSerializableError::from(error))
//     }
// }

// This is a very simple macro that implements a very basic Serializable for each variant of TheseusSerializableError,
// where the field is the string. (This allows easy extension to errors without many match arms)
macro_rules! impl_serialize {
    ($($variant:ident),* $(,)?) => {
        impl Serialize for TheseusSerializableError {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match self {
                    // For the Theseus variant, we add a special display for the error,
                    // to view the spans if subscribed to them (which is information that is lost when serializing)
                    TheseusSerializableError::Theseus(theseus_error) => {
                        $crate::error::display_tracing_error(theseus_error);

                        let mut state = serializer.serialize_struct("Theseus", 2)?;
                        state.serialize_field("field_name", "Theseus")?;
                        state.serialize_field("message", &theseus_error.to_string())?;
                        state.end()
                    }
                    $(
                        TheseusSerializableError::$variant(message) => {
                            let mut state = serializer.serialize_struct(stringify!($variant), 2)?;
                            state.serialize_field("field_name", stringify!($variant))?;
                            state.serialize_field("message", &message.to_string())?;
                            state.end()
                        },
                    )*
                }
            }
        }
    };
}

// Use the macro to implement Serialize for TheseusSerializableError
#[cfg(target_os = "macos")]
impl_serialize! {
    IO,
    Tauri,
    Callback
}

#[cfg(not(target_os = "macos"))]
impl_serialize! {
    IO,
    Tauri,
}
