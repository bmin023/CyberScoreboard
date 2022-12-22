use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io};

use crate::password::{
    get_password_groups, get_passwords, load_password_saves, validate_password_fs, PasswordSave,
};

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
    fn new(name: String, command: String) -> Self {
        Service {
            name,
            command,
            multiplier: 1,
        }
    }
    pub fn is_valid(&self) -> bool {
        return self.name != "" && self.command != "";
    }
    pub fn check_with_env(&self, env: &Vec<(String, String)>) -> io::Result<Output> {
        // get PATH from env
        let path = std::env::var("PATH").unwrap_or("/usr/bin:/bin:/usr/sbin:/sbin".to_string());
        let output = Command::new("sh")
            .current_dir("./resources")
            .arg("-c")
            .arg(&self.command)
            .env_clear()
            .env("PATH", path)
            .envs(env.clone())
            .output();
        output
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Team {
    pub scores: Vec<Score>,
    pub env: Vec<(String, String)>,
}

impl Team {
    pub fn score(&self) -> u32 {
        self.scores.iter().map(|s| s.score).sum()
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SideEffect {
    DeleteService(String),
    AddService(Service),
    EditService(String, Service),
}

impl SideEffect {
    pub fn apply(self, config: &mut Config) {
        match self {
            SideEffect::DeleteService(name) => {
                config.remove_service(&name);
            }
            SideEffect::AddService(service) => {
                config.add_service(service);
            }
            SideEffect::EditService(name, service) => {
                config.edit_service(&name, service);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct YAMLInject {
    file: String,
    start: u32,
    duration: u32,
    side_effects: Option<Vec<SideEffect>>,
}

fn load_injects() -> Vec<Inject> {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub teams: BTreeMap<String, Team>,
    pub services: Vec<Service>,
    pub injects: Vec<Inject>,
    active: bool,
    #[serde(with = "serde_millis")]
    last_start: Instant,
    #[serde(with = "serde_millis")]
    game_time: Duration,
}

impl Config {
    pub fn new() -> Self {
        let services = load_services();
        let teams = load_teams(&services);
        let injects = load_injects();
        let me = Config {
            teams,
            services,
            injects,
            active: true,
            last_start: Instant::now(),
            game_time: Duration::from_secs(0),
        };
        validate_password_fs(&me);
        validate_save_fs();
        me
    }
    pub fn from_save(file_name: &str) -> Result<Self, SaveError> {
        let save = load_save(file_name)?;
        load_password_saves(&save.passwords);
        Ok(save.config)
    }
    pub fn add_team(&mut self, name: String) -> Result<(), TeamError> {
        if name.is_empty() {
            return Err(TeamError::InvalidName);
        }
        if self.teams.contains_key(&name) {
            return Err(TeamError::AlreadyExists);
        }
        self.teams.insert(
            name.clone(),
            Team {
                scores: self.services.iter().map(|_| Score::default()).collect(),
                env: vec![],
            },
        );
        validate_password_fs(self);
        Ok(())
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn run_time(&self) -> Duration {
        if self.active {
            self.game_time + (Instant::now() - self.last_start)
        } else {
            self.game_time
        }
    }
    pub fn stop(&mut self) {
        self.active = false;
        self.game_time += Instant::now() - self.last_start;
    }
    pub fn start(&mut self) {
        self.active = true;
        self.last_start = Instant::now();
    }
    pub fn reset_scores(&mut self) {
        self.active = false;
        self.game_time = Duration::from_secs(0);
        for team in self.teams.values_mut() {
            team.scores = self.services.iter().map(|_| Score::default()).collect();
        }
    }
    pub fn score_tick(&mut self) {
        if self.active {
            let mut side_effects = Vec::new();
            let time = (self.run_time().as_secs() / 60) as u32;
            for inject in self.injects.iter_mut().filter(|i| !i.completed) {
                if time >= inject.start + inject.duration {
                    inject.completed = true;
                    side_effects.extend(inject.side_effects.clone().unwrap_or_default());
                }
            }
            for effect in side_effects {
                println!("Applying side effect: {:?}", effect);
                effect.apply(self);
            }
            score_teams(self);
        }
    }
    pub fn open_injects(&self) -> Vec<Inject> {
        let time = (self.run_time().as_secs() / 60) as u32;
        let injects = self
            .injects
            .iter()
            .filter(|i| i.start >= time)
            .cloned()
            .collect();
        injects
    }
    pub fn remove_service(&mut self, name: &str) {
        if let Some(index) = self.services.iter().position(|s| s.name == name) {
            self.services.remove(index);
            for team in self.teams.values_mut() {
                team.scores.remove(index);
            }
        }
    }
    pub fn add_service(&mut self, service: Service) {
        self.services.push(service);
        for team in self.teams.values_mut() {
            team.scores.push(Score::default());
        }
    }
    pub fn edit_service(&mut self, name: &str, service: Service) {
        for s in self.services.iter_mut() {
            if s.name == name {
                *s = service.clone();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Save {
    #[serde(with = "serde_millis")]
    pub saved_at: Instant,
    config: Config,
    /// Map with the key being the team name and the value being a vector of
    /// their passwords
    passwords: BTreeMap<String, Vec<PasswordSave>>,
}

pub enum SaveError {
    ReadError,
    ParseError,
    WriteError,
}

pub fn save_config(config: &Config, file_name: &str) -> Result<(), SaveError> {
    let saved_at = Instant::now();
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
        saved_at,
        config: config.clone(),
        passwords,
    };
    let Ok(file) = fs::File::create(format!("resources/save/{}.json", file_name)) else {
        println!("Error opening save file");
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
    println!("Autosaving to {}", time);
    save_config(config, format!("autosave/autosave-{}", time).as_str())
}

pub fn load_save(file_name: &str) -> Result<Save, SaveError> {
    let Ok(file) = fs::File::open(format!("resources/save/{}.json", file_name)) else {
        println!("Error opening save file");
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

fn validate_save_fs() {
    // make sure the save directory exists
    if let Err(e) = fs::create_dir_all("resources/save/autosave") {
        println!("Error creating save directory: {}", e);
    }
}

fn load_teams(services: &Vec<Service>) -> BTreeMap<String, Team> {
    let file = fs::read_to_string("resources/teams.yaml")
        .expect("teams.yaml should be in the resources directory");
    let teams = serde_yaml::from_str::<BTreeMap<String, BTreeMap<String, String>>>(&file)
        .expect("teams.yaml should be correctly formatted");
    let teams = teams
        .iter()
        .map(|(name, env)| {
            let env = env
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<(String, String)>>();
            (
                name.clone(),
                Team {
                    scores: services.iter().map(|_| Score::default()).collect(),
                    env,
                },
            )
        })
        .collect::<BTreeMap<String, Team>>();
    teams
}

fn load_services() -> Vec<Service> {
    let file = fs::read_to_string("resources/services.yaml")
        .expect("service.yaml should be in the resources directory");
    let commands = serde_yaml::from_str::<BTreeMap<String, String>>(&file)
        .expect("service.yaml should be correctly formatted");
    let mut services = Vec::new();
    for (name, command) in commands {
        services.push(Service::new(name, command));
    }
    services
}

fn score_teams(config: &mut Config) {
    let services = &config.services;
    let exit_codes = Arc::new(Mutex::new(BTreeMap::<String, Vec<bool>>::new()));
    for team in config.teams.keys() {
        exit_codes
            .lock()
            .unwrap()
            .insert(team.clone(), vec![false; services.len()]);
    }
    std::thread::scope(|s| {
        for (name, team) in &config.teams {
            for (i, check) in services.iter().enumerate() {
                let codes = Arc::clone(&exit_codes);
                s.spawn(move || {
                    let Ok(output) = check.check_with_env(&team.env) else {
                        return;
                    };
                    if output.status.success() {
                        let mut codes = codes.lock().unwrap();
                        let codevec = codes
                            .entry(name.clone())
                            .or_insert(vec![false; services.len()]);
                        codevec[i] = true;
                    }
                });
            }
        }
    });
    let codes = exit_codes.lock().unwrap();
    for (team_name, team_codes) in codes.iter() {
        config.teams.entry(team_name.clone()).and_modify(|team| {
            for (i, code) in team_codes.iter().enumerate() {
                let up = *code;
                team.scores[i].up = up;
                if up {
                    team.scores[i].score += services[i].multiplier as u32;
                }
                team.scores[i].history.push_front(up);
                if team.scores[i].history.len() > 10 {
                    team.scores[i].history.pop_back();
                }
                println!(
                    "{} {}: {}",
                    team_name, services[i].name, &team.scores[i].score
                );
            }
        });
    }
}
