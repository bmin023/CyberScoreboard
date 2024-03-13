use std::{collections::BTreeMap, fs, io::Write, time::SystemTime};
use uuid::Uuid;

use handlebars::Handlebars;
use markdown::to_html;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::checker::resource_location;

use super::{Config, config::ConfigError, Service};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Inject {
    pub uuid: Uuid,
    pub name: String,
    pub markdown: String,
    /// Time when the inject happens in minutes
    pub start: u32,
    /// Duration of inject in minutes
    pub duration: u32,
    pub side_effects: Vec<SideEffect>,
    pub completed: bool,
    /// If empty, no file types are allowed. If None, All file types
    /// are allowed.
    pub file_type: Option<Vec<String>>,
    pub sticky: bool,
}

fn team_inject_dir(team_name: &str) -> String {
    format!("{}/injects/{}", resource_location(), team_name)
}

impl Inject {
    pub fn new(inject: CreateInject) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: inject.name,
            markdown: inject.markdown,
            start: inject.start,
            duration: inject.duration,
            side_effects: inject.side_effects,
            completed: false,
            file_type: inject.file_type,
            sticky: inject.sticky,
        }
    }
    pub fn is_active(&self, minutes_since_start: u32) -> bool {
        minutes_since_start >= self.start && minutes_since_start < self.start + self.duration
    }
    pub fn is_ended(&self, minutes_since_start: u32) -> bool {
        minutes_since_start >= self.start + self.duration
    }
    pub fn requires_response(&self) -> bool {
        if let Some(f) = &self.file_type {
            !f.is_empty()
        } else {
            true
        }
    }
    fn from_yaml(name: String, yaml: YAMLInject) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            markdown: yaml.markdown,
            start: yaml.start,
            duration: yaml.duration.unwrap_or(1),
            side_effects: yaml.side_effects.unwrap_or(vec![]),
            completed: false,
            file_type: if yaml.no_submit {
                Some(vec![])
            } else {
                yaml.file_types
            },
            sticky: yaml.duration.is_none(),
        }
    }
    fn format_name(&self) -> String {
        self.name.replace(" ", "_")
    }
    /// Creates a new file in resources/injects/<team_name>/filename
    /// Then sends back an artifact that the team did in fact submit.
    pub fn new_response(
        &self,
        team_name: &str,
        filename: &str,
        data: &[u8],
    ) -> Result<InjectResponse, ResponseError> {
        // check if folder exists
        let path = team_inject_dir(team_name);
        fs::create_dir_all(path).map_err(|_| ResponseError::FileError)?;
        let extension = match filename.split(".").last() {
            Some(ext) => format!(".{}", ext),
            None => "".to_string(),
        };
        let new_filename = if self.completed {
            format!("{}_late_response{}", self.format_name(), extension)
        } else {
            format!("{}_response{}", self.format_name(), extension)
        };
        let path = format!("{}/{}", team_inject_dir(team_name), new_filename);
        let mut file = fs::File::create(path).map_err(|_| ResponseError::FileError)?;
        file.write_all(data).map_err(|_| ResponseError::FileError)?;
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Ok(InjectResponse {
            uuid: Uuid::new_v4(),
            inject_uuid: self.uuid.clone(),
            late: self.completed,
            filename: filename.to_string(),
            upload_time: time,
            name: self.name.clone(),
        })
    }
    pub fn get_html(&self, env: &Vec<(String, String)>) -> String {
        let map: BTreeMap<String, String> = env.iter().cloned().collect();
        let reg = Handlebars::new();
        let filled_md = reg
            .render_template(&self.markdown, &map)
            .unwrap_or(self.markdown.clone());
        to_html(&filled_md)
    }
}

pub enum ResponseError {
    NoName,
    FileError,
    InjectNotFound,
    TeamNotFound,
}

pub fn load_injects() -> Vec<Inject> {
    let inject_file = std::env::var("SB_INJECTS").unwrap_or("injects.yaml".to_string());
    let Ok(file) = fs::read_to_string(format!("{}/{}", resource_location(), inject_file)) else {
        return Vec::new();
    };
    let yaml_tree: BTreeMap<String, YAMLInject> =
        serde_yaml::from_str(&file).expect(format!("{} is not valid", inject_file).as_str());
    let injects: Vec<Inject> = yaml_tree
        .into_iter()
        .map(|(name, inject)| Inject::from_yaml(name, inject))
        .collect();
    info!("Loaded {} injects", injects.len());
    debug!("Injects: {:?}", injects);
    injects
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SideEffect {
    DeleteService(String),
    AddService(Service),
    EditService(String, Service),
}

impl SideEffect {
    pub fn apply(self, config: &mut Config) -> Result<(), ConfigError> {
        match self {
            SideEffect::DeleteService(name) => config.remove_service(&name),
            SideEffect::AddService(service) => config.add_service(service),
            SideEffect::EditService(name, service) => config.edit_service(&name, service),
        }
    }
}

#[derive(Deserialize)]
struct YAMLInject {
    markdown: String,
    file_types: Option<Vec<String>>,
    #[serde(default)]
    start: u32,
    duration: Option<u32>,
    side_effects: Option<Vec<SideEffect>>,
    #[serde(default)]
    no_submit: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InjectResponse {
    pub inject_uuid: Uuid,
    pub name: String,
    pub uuid: Uuid,
    pub late: bool,
    pub filename: String,
    pub upload_time: u128,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CreateInject {
    pub name: String,
    pub markdown: String,
    pub start: u32,
    pub duration: u32,
    pub side_effects: Vec<SideEffect>,
    pub file_type: Option<Vec<String>>,
    #[serde(default)]
    pub sticky: bool,
}

pub trait InjectUser {
    fn inject_tick(&mut self);
    fn submit_response(
        &mut self,
        team_name: &str,
        inject_uuid: Uuid,
        filename: &str,
        data: &[u8],
    ) -> Result<(), ResponseError>;
    fn get_inject(&self, inject_uuid: Uuid) -> Option<Inject>;
    fn get_injects_for_team(&self, team: &str) -> Result<Vec<Inject>, ConfigError>;
    fn add_inject(&mut self, inject: CreateInject);
    fn edit_inject(&mut self, inject: Inject) -> Result<(), ConfigError>;
    fn delete_inject(&mut self, inject_uuid: Uuid) -> Result<(), ConfigError>;
}

impl InjectUser for Config {
    fn inject_tick(&mut self) {
        let mut side_effects = Vec::new();
        let time = (self.run_time().as_secs() / 60) as u32;
        for inject in self
            .injects
            .iter_mut()
            .filter(|i| !i.sticky && !i.completed)
        {
            if inject.is_ended(time) {
                info!("Inject {} has ended", inject.name);
                inject.completed = true;
                side_effects.extend(inject.side_effects.clone());
            }
        }
        for effect in side_effects {
            info!("Applying side effect: {:?}", effect);
            if let Err(err) = effect.apply(self) {
                error!("Error applying side effect: {:?}", err);
            }
        }
    }
    fn submit_response(
        &mut self,
        team_name: &str,
        inject_uuid: Uuid,
        filename: &str,
        data: &[u8],
    ) -> Result<(), ResponseError> {
        if let Some(team) = self.teams.get_mut(team_name) {
            if let Some(inject) = self.injects.iter_mut().find(|i| i.uuid == inject_uuid) {
                let res = inject.new_response(team_name, filename, data)?;
                team.inject_responses.push(res);
                Ok(())
            } else {
                Err(ResponseError::InjectNotFound)
            }
        } else {
            Err(ResponseError::TeamNotFound)
        }
    }
    fn get_inject(&self, inject_uuid: Uuid) -> Option<Inject> {
        self.injects.iter().find(|i| i.uuid == inject_uuid).cloned()
    }
    fn get_injects_for_team(&self, team: &str) -> Result<Vec<Inject>, ConfigError> {
        let team = self.teams.get(team).ok_or(ConfigError::DoesNotExist)?;
        let time = (self.run_time().as_secs() / 60) as u32;
        Ok(self
            .injects
            .iter()
            .filter(|i| {
                i.is_active(time) || (i.is_ended(time) && (i.sticky || (i.requires_response() && !team.has_response(i.uuid))))
            })
            .cloned()
            .collect())
    }
    fn add_inject(&mut self, inject: CreateInject) {
        let inject = Inject::new(inject);
        self.injects.push(inject);
    }
    fn edit_inject(&mut self, inject: Inject) -> Result<(), ConfigError> {
        let index = self
            .injects
            .iter()
            .position(|i| i.uuid == inject.uuid)
            .ok_or(ConfigError::DoesNotExist)?;
        self.injects[index] = inject;
        // reevaluate completed
        self.injects[index].completed = if self.injects[index].sticky {
            false
        } else {
            self.injects[index].is_ended((self.run_time().as_secs() / 60) as u32)
        };
        Ok(())
    }
    fn delete_inject(&mut self, inject_uuid: Uuid) -> Result<(), ConfigError> {
        let index = self
            .injects
            .iter()
            .position(|i| i.uuid == inject_uuid)
            .ok_or(ConfigError::DoesNotExist)?;
        self.injects.remove(index);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_html() {
        let inject = Inject {
            uuid: Uuid::new_v4(),
            name: "Test Inject".to_string(),
            markdown: "This is a test inject {{VARIABLE}}".to_string(),
            start: 0,
            duration: 0,
            side_effects: vec![],
            completed: false,
            file_type: None,
            sticky: false,
        };
        let vec = &vec![("VARIABLE".to_string(), "test".to_string())];
        let html = inject.get_html(vec);
        assert_eq!(html, "<p>This is a test inject test</p>\n");
    }
    #[test]
    fn check_json_injects() {
        let inject = Inject {
            uuid: Uuid::new_v4(),
            name: "Test Inject".to_string(),
            markdown: "This is a test inject {{VARIABLE}}".to_string(),
            start: 0,
            duration: 0,
            side_effects: vec![
                SideEffect::EditService(
                    "test".to_string(),
                    Service {
                        name: "test".to_string(),
                        command: "test".to_string(),
                        multiplier: 1,
                    },
                ),
                SideEffect::DeleteService("test".to_string()),
                SideEffect::AddService(Service {
                    name: "test".to_string(),
                    command: "test".to_string(),
                    multiplier: 1,
                }),
            ],
            completed: false,
            file_type: None,
            sticky: false,
        };
        println!("{}", serde_json::to_string(&inject).unwrap());
    }
}
