use std::{collections::BTreeMap, fs, io::Write, time::SystemTime};
use uuid::Uuid;

use handlebars::Handlebars;
use markdown::to_html;

use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use super::{Config, ConfigError, Service};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Inject {
    pub uuid: Uuid,
    pub name: String,
    pub markdown: String,
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
            uuid: Uuid::new_v4(),
            name,
            markdown: yaml.markdown,
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
        filename: &str,
        data: &[u8],
    ) -> Result<InjectResponse, ResponseError> {
        // check if folder exists
        let path = format!("resources/injects/{}", team_name);
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
        let path = format!("resources/injects/{}/{}", team_name, new_filename);
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
    let Ok(file) = fs::read_to_string("resources/injects.yaml") else {
        return Vec::new();
    };
    let yaml_tree: BTreeMap<String, YAMLInject> =
        serde_yaml::from_str(&file).expect("injects.yaml is not valid");
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
    file_type: Option<Vec<String>>,
    start: u32,
    duration: u32,
    side_effects: Option<Vec<SideEffect>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InjectResponse {
    pub inject_uuid: Uuid,
    pub uuid: Uuid,
    pub late: bool,
    pub filename: String,
    pub upload_time: u128,
}

// tests for html
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
            side_effects: None,
            completed: false,
            file_type: None,
        };
        let vec = &vec![("VARIABLE".to_string(), "test".to_string())];
        let html = inject.get_html(vec);
        assert_eq!(html, "<p>This is a test inject test</p>\n");
    }
}

