//! API for interacting with Theseus
pub mod cache;
pub mod handler;
pub mod jre;
pub mod logs;
pub mod metadata;
pub mod minecraft_auth;
pub mod mr_auth;
pub mod pack;
pub mod process;
pub mod profile;
pub mod settings;
pub mod skin_manager;
pub mod tags;

pub mod data {
    pub use crate::state::{
        Credentials, Dependency, DirectoryInfo, Hooks, JavaVersion, LinkedData,
        MemorySettings, ModLoader, ModrinthCredentials,
        ModrinthCredentialsResult, Organization, Process, ProfileFile, Project,
        ProjectType, SearchResult, SearchResults, Settings, TeamMember, Theme,
        User, Version, WindowSize,
    };
}

pub mod prelude {
    pub use crate::{
        data::*,
        event::CommandPayload,
        jre, metadata, minecraft_auth, mr_auth, pack, process,
        profile::{self, create, Profile},
        settings,
        util::io::{canonicalize, IOError},
        State,
    };
}
