use std::{str::FromStr, collections::BTreeMap};

use serde::{Serialize, Deserialize};
use tracing::{error, info};

use crate::checker::Config;

use super::resource_location;

#[derive(Debug)]
pub struct UserPass {
    pub username: String,
    pub password: String,
}
impl ToString for UserPass {
    fn to_string(&self) -> String {
        format!("{}:{}", self.username, self.password)
    }
}

impl FromStr for UserPass {
    type Err = PasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(':');
        let username = split.next().unwrap_or("");
        let password = split.next().unwrap_or("");
        if username.is_empty() || password.is_empty() {
            return Err(PasswordError::ParseError);
        }
        let regex = regex::Regex::new(r"^[a-zA-Z0-9!@#?%&]+$").unwrap();
        if !regex.is_match(username) || !regex.is_match(password) {
            return Err(PasswordError::ParseError);
        }
        Ok(UserPass {
            username: username.to_string(),
            password: password.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasswordSave {
    pub group: String,
    pub passwords: String
}

pub fn load_password_saves(save: &BTreeMap<String,Vec<PasswordSave>>) {
    // clear the PW directory
    if let Err(err) = std::fs::remove_dir_all(password_dir()) {
        error!("Error removing directory {}: {}",password_dir(), err);
    }
    if let Err(err) = std::fs::create_dir(password_dir()) {
        error!("Error creating directory {}: {}", password_dir(), err);
    }

    for (team,save) in save.iter() {
        // create the team directory
        if let Err(err) = std::fs::create_dir(team_password_dir(team)) {
            error!("Error creating directory {}: {}", team_password_dir(team), err);
        }
        for password in save {
            if let Err(err) = write_passwords(team, &password.group, &password.passwords) {
                error!("Error writing passwords: {:?}", err);
            }
        }
    }
}

#[derive(Debug)]
pub enum PasswordError {
    InvalidFile,
    ParseError,
}

pub fn validate_password_fs(config: &Config) {
    let mut path = std::path::PathBuf::from(password_dir());
    if !path.exists() {
        std::fs::create_dir(&path).unwrap();
    }
    for (team,_) in config.teams.iter() {
        path.push(&team);
        if !path.exists() {
            std::fs::create_dir(&path).unwrap();
        }
        path.pop();
    }
    let Ok(read_dir) = std::fs::read_dir(&path) else {
        error!("Error reading directory password directory");
        return;
    };
    // loop through path, if team doesn't exist, remove it
    for entry in read_dir {
        if let Ok(entry) = entry {
            if let Some(filename) = entry.file_name().to_str() {
                if !config.teams.contains_key(filename) {
                    if let Err(err) = std::fs::remove_dir_all(team_password_dir(&filename.to_string())) {
                        error!("Error removing directory: {}", err);
                    } else {
                        info!("Removed directory of nonexistent team: {}", filename);
                    }
                }
            }
        }
    }
}

pub fn remove_password_group(team: &String, group: &String) -> Result<(), PasswordError> {
    let path = format!("{}/{}.pw", team_password_dir(team), group);
    std::fs::remove_file(path).map_err(|_| PasswordError::InvalidFile)?;
    Ok(())
}

pub fn get_password_groups(team: &String) -> Result<Vec<String>, PasswordError> {
    let path = team_password_dir(team);
    let mut groups = Vec::new();
    if let Ok(dir) = std::fs::read_dir(path) {
        for entry in dir {
            if let Ok(entry) = entry {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".pw") {
                        // remove .pw
                        groups.push(filename[..filename.len() - 3].to_string());
                    }
                }
            }
        }
        Ok(groups)
    } else {
        Err(PasswordError::InvalidFile)
    }
}

pub fn get_passwords(team: &String, group: &String) -> Result<String, PasswordError> {
    let path = format!("{}/{}.pw", team_password_dir(team), group);
    let contents = std::fs::read_to_string(path).map_err(|_| PasswordError::InvalidFile)?;
    Ok(contents)
}

fn read_passwords(
    team_name: &String,
    group: &String,
) -> Result<Vec<UserPass>, PasswordError> {
    // Read the file at resources/PW/<team_name>/<password_file>.pw
    let path = format!("{}/{}.pw", team_password_dir(team_name), group);
    let contents = std::fs::read_to_string(path).map_err(|_| PasswordError::InvalidFile)?;
    let passwords = parse_passwords(&contents);
    Ok(passwords)
}

pub fn write_passwords(
    team_name: &String,
    group: &String,
    passwords: &String,
) -> Result<(), PasswordError> {
    let path = format!("{}/{}.pw", team_password_dir(team_name), group);
    // I know this looks stupid.
    // But we want to parse the passwords to make sure they are valid before we write them to the file.
    let contents = passwords_to_string(&parse_passwords(&passwords));
    std::fs::write(path, contents).map_err(|_| PasswordError::InvalidFile)?;
    Ok(())
}

pub fn overwrite_passwords(
    team_name: &String,
    group: &String,
    passwords: &String,
) -> Result<(), PasswordError> {
    let path = format!("{}/{}.pw",team_password_dir(team_name), group);
    let mut old_passwords = read_passwords(team_name, group)?;
    for password in parse_passwords(passwords) {
        if let Some(index) = old_passwords.iter().position(|p| p.username == password.username) {
            old_passwords[index] = password;
        }
    };
    std::fs::write(path,passwords_to_string(&old_passwords)).map_err(|_| PasswordError::InvalidFile)?;
    Ok(())
}

/// Parses a string of the form "username:password" into a UserPass struct.
/// Returns an error if any of the strings are not valid.
fn parse_passwords(password_string: &String) -> Vec<UserPass> {
    let passwords = password_string.split_whitespace().filter_map(|s| s.parse().ok()).collect();
    passwords
}

fn passwords_to_string(passwords: &Vec<UserPass>) -> String {
    let mut password_string = String::new();
    for password in passwords {
        password_string.push_str(&password.to_string());
        password_string.push_str("\n");
    }
    password_string
}

fn password_dir() -> String {
    format!("{}/PW", resource_location())
}

fn team_password_dir(team: &String) -> String {
    format!("{}/{}",password_dir(),team)
}
