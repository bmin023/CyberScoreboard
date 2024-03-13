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

pub use self::config::Config;
pub use self::{
    service::Service,
    team::{Score, Team, TeamError},
};


pub fn resource_location() -> String {
    std::env::var("SB_RESOURCE_DIR").unwrap_or_else(|_| "resources".to_string())
}
