use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    process::Command,
};
use wait_timeout::ChildExt;

use serde::{Deserialize, Serialize};

use super::{Config, ConfigError};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Score {
    pub score: u32,
    pub up: bool,
    pub history: VecDeque<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Service {
    pub name: String,
    pub command: String,
    pub multiplier: u8,
}
impl Service {
    pub fn new(name: String, command: String) -> Self {
        Service {
            name,
            command,
            multiplier: 1,
        }
    }
    pub fn is_valid(&self) -> bool {
        return self.name != "" && self.command != "";
    }
    pub fn check_with_env(&self, env: &Vec<(String, String)>) -> Result<TestOutput, ()> {
        // get PATH from env
        let path = std::env::var("PATH").unwrap_or("/usr/bin:/bin:/usr/sbin:/sbin".to_string());
        let child = Command::new("sh")
            .current_dir("./resources")
            .arg("-c")
            .arg(&self.command)
            .env_clear()
            .env("PATH", path)
            .envs(env.clone())
            .spawn();
        let mut child = match child {
            Ok(child) => child,
            Err(_) => return Err(()),
        };
        let output = child.wait_timeout(std::time::Duration::from_secs(5));
        let output = match output {
            Ok(output) => output,
            Err(_) => return Err(()),
        };
        match output {
            Some(status) => Ok(TestOutput {
                up: status.success(),
                message: "I killed stdout I swear it will be back".to_string(),
                error: "I killed stderr I swear it will be back".to_string(),
            }),
            None => {
                child.kill().unwrap();
                Ok(TestOutput {
                    up: false,
                    message: "".to_string(),
                    error: "Timeout".to_string(),
                })
            }
        }
    }
}

pub struct TestOutput {
    pub up: bool,
    pub message: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Team {
    pub scores: BTreeMap<String, Score>,
    pub env: Vec<(String, String)>,
}

impl Team {
    pub fn score(&self) -> u32 {
        self.scores.iter().map(|(_, s)| s.score).sum()
    }
}

pub enum TeamError {
    InvalidName,
    AlreadyExists,
}

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
