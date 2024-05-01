mod artifact;
pub mod config;
mod inject;
mod password;
mod save;
mod service;
mod team;

pub mod saves {
    pub use super::save::{get_autosave_names, get_save_names, load_save, SaveError};
}
pub mod passwords {
    pub use super::password::{
        get_password_groups, get_passwords, overwrite_passwords, remove_password_group,
        write_passwords, PasswordSave,
    };
}
pub mod injects {
    pub use super::inject::{CreateInject, Inject, InjectResponse, InjectUser};
}

use std::time::SystemTime;

use serde::Serialize;

pub use self::config::Config;
pub use self::{
    service::Service,
    team::{Score, Team, TeamError},
};

pub fn resource_location() -> String {
    std::env::var("SB_RESOURCE_DIR").unwrap_or_else(|_| "resources".to_string())
}

#[derive(Serialize)]
pub struct ScoreboardInfo {
    pub version: String,
}

impl Default for ScoreboardInfo {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

pub fn current_time() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
