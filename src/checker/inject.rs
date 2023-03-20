use std::{fs, collections::BTreeMap, io::Write, error::Error};

use serde::{Serialize, Deserialize};

use super::{Service, Config, ConfigError};

#[derive(Clone, Serialize, Deserialize)]
pub struct Inject {
    pub name: String,
    pub file: String,
    /// Time when the inject happens in seconds
    pub start: u32,
    /// Duration of inject in seconds
    pub duration: u32,
    pub side_effects: Option<Vec<SideEffect>>,
    pub completed: bool,
}

impl Inject {
    fn from_yaml(name: String, yaml: YAMLInject) -> Self {
        Self {
            name,
            file: yaml.file,
            start: yaml.start,
            duration: yaml.duration,
            side_effects: yaml.side_effects,
            completed: false,
        }
    }
    fn format_name(&self) -> String {
        self.name.replace(" ", "_")
    }
    /// Creates a new file in at resources/injects/<team_name>/filename
    /// Then sends back an artifact that the team did in fact submit.
    pub fn new_response(&self, team_name: &str, extension: &str, data: &[u8]) -> Result<InjectResponse,ResponseError> {
        // check if folder exists
        let path = format!("resources/injects/{}", team_name);
        fs::create_dir_all(path).map_err(|_| ResponseError::FileError)?;
        let filename = format!("{}_response.{}", self.format_name(), extension);
        let path = format!("resources/injects/{}/{}", team_name, filename);
        let mut file = fs::File::create(path).map_err(|_| ResponseError::FileError)?;
        file.write_all(data).map_err(|_| ResponseError::FileError)?;
        Ok(InjectResponse {
            name: self.name.clone(),
            late: !self.completed,
            filename: filename.to_string(),
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
    let injects = yaml_tree
        .into_iter()
        .map(|(name, inject)| Inject::from_yaml(name, inject))
        .collect();
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
    start: u32,
    duration: u32,
    side_effects: Option<Vec<SideEffect>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InjectResponse {
    pub name: String,
    pub late: bool,
    pub filename: String,
}
