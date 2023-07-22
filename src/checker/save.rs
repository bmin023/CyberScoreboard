use std::{time::{SystemTime, UNIX_EPOCH}, collections::BTreeMap, fs};

use serde::{Serialize, Deserialize};
use tracing::error;

use super::passwords::{PasswordSave, get_password_groups, get_passwords};

use super::Config;

#[derive(Serialize, Deserialize)]
pub struct Save {
    pub saved_at: u128,
    pub config: Config,
    /// Map with the key being the team name and the value being a vector of
    /// their passwords
    pub passwords: BTreeMap<String, Vec<PasswordSave>>,
}

#[derive(Debug)]
pub enum SaveError {
    ReadError,
    ParseError,
    WriteError,
    InternalError
}

pub fn save_config(config: &Config, file_name: &str) -> Result<(), SaveError> {
    let Ok(saved_at) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return Err(SaveError::InternalError);
    };
    let passwords = config
        .teams
        .iter()
        .filter_map(|(name, _)| {
            if let Ok(groups) = get_password_groups(&name) {
                let saves = groups
                    .iter()
                    .filter_map(|group| {
                        if let Ok(passwords) = get_passwords(&name, &group) {
                            Some(PasswordSave {
                                group: group.clone(),
                                passwords,
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<PasswordSave>>();
                Some((name.clone(), saves))
            } else {
                None
            }
        })
        .collect();
    let save = Save {
        saved_at: saved_at.as_millis(),
        config: config.clone(),
        passwords,
    };
    let Ok(file) = fs::File::create(format!("resources/save/{}.json", file_name)) else {
        error!("Error opening save file resources/save/{}.json",file_name);
        return Err(SaveError::WriteError);
    };
    match serde_json::to_writer(file, &save) {
        Ok(_) => Ok(()),
        Err(_) => Err(SaveError::ParseError),
    }
}

pub fn autosave(config: &Config) -> Result<(), SaveError> {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 60
        % 12;
    save_config(config, format!("autosave/autosave-{}", time).as_str())
}

pub fn load_save(file_name: &str) -> Result<Save, SaveError> {
    let Ok(file) = fs::File::open(format!("resources/save/{}.json", file_name)) else {
        error!("Error opening save file resources/save/{}.json",file_name);
        return Err(SaveError::ReadError);
    };
    match serde_json::from_reader(file) {
        Ok(save) => Ok(save),
        Err(_) => Err(SaveError::ParseError),
    }
}

pub fn get_save_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir("resources/save") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                names.push(name[0..name.len() - 5].to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    names
}

pub fn get_autosave_names() -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir("resources/save/autosave") {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                names.push(name[0..name.len() - 5].to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    names
}

pub fn validate_save_fs() {
    // make sure the save directory exists
    if let Err(e) = fs::create_dir_all("resources/save/autosave") {
        error!("Error creating save directory resources/save/autosave: {}", e);
    }
}