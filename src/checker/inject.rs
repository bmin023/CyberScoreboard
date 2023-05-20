use std::{collections::BTreeMap, fs, io::Write, time::SystemTime};

use serde::{Deserialize, Serialize};
use tracing::info;

use super::{Config, ConfigError, Service};

#[derive(Clone, Serialize, Deserialize)]
pub struct Inject {
    pub name: String,
    pub file: String,
    /// Time when the inject happens in minutes
    pub start: u32,
    /// Duration of inject in minutes
    pub duration: u32,
    pub side_effects: Option<Vec<SideEffect>>,
    pub completed: bool,
    pub file_type: Option<Vec<String>>,
}

impl Inject {
    pub fn is_active(&self, minutes_since_start: u32) -> bool {
        minutes_since_start >= self.start && minutes_since_start < self.start + self.duration
    }
    fn from_yaml(name: String, yaml: YAMLInject) -> Self {
        Self {
            name,
            file: yaml.file,
            start: yaml.start,
            duration: yaml.duration,
            side_effects: yaml.side_effects,
            completed: false,
            file_type: yaml.file_type,
        }
    }
    fn format_name(&self) -> String {
        self.name.replace(" ", "_")
    }
    /// Creates a new file in at resources/injects/<team_name>/filename
    /// Then sends back an artifact that the team did in fact submit.
    pub fn new_response(
        &self,
        team_name: &str,
        extension: &str,
        data: &[u8],
    ) -> Result<InjectResponse, ResponseError> {
        // check if folder exists
        let path = format!("resources/injects/{}", team_name);
        fs::create_dir_all(path).map_err(|_| ResponseError::FileError)?;
        let filename = if self.completed {
            format!("{}_late_response.{}", self.format_name(), extension)
        } else {
            format!("{}_response.{}", self.format_name(), extension)
        };
        let path = format!("resources/injects/{}/{}", team_name, filename);
        let mut file = fs::File::create(path).map_err(|_| ResponseError::FileError)?;
        file.write_all(data).map_err(|_| ResponseError::FileError)?;
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(InjectResponse {
            name: self.name.clone(),
            late: !self.completed,
            filename: filename.to_string(),
            upload_time: time,
        })
    }
}

pub enum ResponseError {
    NoName,
    FileError,
    InjectNotFound,
    TeamNotFound,
}

pub fn load_injects() -> Vec<Inject> {
    let Ok(file) = fs::read_to_string("resources/injects.yaml") else {
        return Vec::new();
    };
    let yaml_tree: BTreeMap<String, YAMLInject> =
        serde_yaml::from_str(&file).expect("injects.yaml is not valid");
    let injects : Vec<Inject> = yaml_tree
        .into_iter()
        .map(|(name, inject)| Inject::from_yaml(name, inject))
        .collect();
    info!("Loaded {} injects", injects.len());
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
    file: String,
    file_type: Option<Vec<String>>,
    start: u32,
    duration: u32,
    side_effects: Option<Vec<SideEffect>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InjectResponse {
    pub name: String,
    pub late: bool,
    pub filename: String,
    pub upload_time: u64,
}
