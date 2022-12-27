mod base;
mod password;
mod save;
pub mod saves {
    pub use super::save::{get_autosave_names, get_save_names, load_save,SaveError};
}
pub mod passwords {
    pub use super::password::{
        get_password_groups, get_passwords, overwrite_passwords, remove_password_group,
        write_passwords, PasswordSave,
    };
}

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::{debug, error, info};

use std::fmt::Display;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base::*;
pub use base::{Score, Service, Team, TeamError};
use password::{load_password_saves, validate_password_fs};
use save::{autosave, load_save, save_config, validate_save_fs, SaveError};

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
    // #[serde(skip)]
    // to_delete: Vec<String>,
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
            // to_delete: vec![],
        };
        validate_password_fs(&me);
        validate_save_fs();
        me
    }
    pub fn save(&self, file_name: &str) -> Result<(), SaveError> {
        save_config(self, file_name)
    }
    pub fn autosave(&self) -> Result<(), SaveError> {
        autosave(&self)
    }
    pub fn from_save(file_name: &str) -> Result<Self, SaveError> {
        let mut save = load_save(file_name)?;
        load_password_saves(&save.passwords);
        save.config.active = false;
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
                scores: self
                    .services
                    .iter()
                    .map(|s| (s.name.to_owned(), Score::default()))
                    .collect(),
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
            team.scores = self
                .services
                .iter()
                .map(|s| (s.name.to_owned(), Score::default()))
                .collect();
        }
    }
    pub fn inject_tick(&mut self) {
        let mut side_effects = Vec::new();
        let time = (self.run_time().as_secs() / 60) as u32;
        for inject in self.injects.iter_mut().filter(|i| !i.completed) {
            if time >= inject.start + inject.duration {
                inject.completed = true;
                side_effects.extend(inject.side_effects.clone().unwrap_or_default());
            }
        }
        for effect in side_effects {
            info!("Applying side effect: {:?}", effect);
            if let Err(err) = effect.apply(self) {
                error!("Error applying side effect: {:?}", err);
            }
        }
    }
    pub fn score_tick(&mut self) {
        score_teams(self);
    }
    pub fn remove_service(&mut self, name: &str) -> Result<(), ConfigError> {
        if let Some(index) = self.services.iter().position(|s| s.name == name) {
            self.services.remove(index);
            Ok(())
        } else {
            Err(ConfigError::DoesNotExist)
        }
    }
    pub fn add_service(&mut self, service: Service) -> Result<(), ConfigError> {
        if let Some(_) = self.services.iter().find(|s| s.name == service.name) {
            return Err(ConfigError::AlreadyExists);
        }
        if !service.is_valid() {
            return Err(ConfigError::BadValue);
        }
        for team in self.teams.values_mut() {
            team.scores.insert(service.name.clone(), Score::default());
        }
        self.services.push(service);
        Ok(())
    }
    pub fn edit_service(&mut self, name: &str, service: Service) -> Result<(), ConfigError> {
        if !service.is_valid() {
            return Err(ConfigError::BadValue);
        }
        if service.name != name && self.services.iter().any(|s| s.name == service.name) {
            return Err(ConfigError::AlreadyExists);
        }
        if name != service.name {
            for team in self.teams.values_mut() {
                let score = team.scores.remove(name);
                team.scores
                    .insert(service.name.clone(), score.unwrap_or_default());
                // self.to_delete.push(name.to_owned());
            }
        }
        for s in self.services.iter_mut() {
            if s.name == name {
                *s = service;
                return Ok(());
            }
        }
        Ok(())
    }
    /// Because there can technically be multiple sources of truth for the config,
    /// this function will combine the two configs together, with this config
    /// taking precedence. The other config will try and update this one while respecting
    /// new services and teams.
    #[tracing::instrument(skip(self, other))]
    pub fn smart_combine(&mut self, other: Config) {
        for (team_name, other_team) in other.teams {
            self.teams.entry(team_name).and_modify(|team| {
                for (new_score_name, new_score) in other_team.scores {
                    if team.scores.contains_key(&new_score_name)
                    // || !self.to_delete.contains(&new_score_name)
                    {
                        team.scores.insert(new_score_name, new_score);
                    }
                }
            });
        }
        // self.to_delete.clear();
        // update injects
        for inject in other.injects {
            if let Some(index) = self.injects.iter().position(|i| i.name == inject.name) {
                self.injects[index] = inject;
            } else {
                debug!("Couldn't resolve inject: {}", inject.name);
            }
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // game time
        let game_time = self.run_time();
        let game_time = format!("{}:{}", game_time.as_secs() / 60, game_time.as_secs() % 60);
        writeln!(f, "Game time: {}", game_time)?;
        // teams
        for (name, team) in self.teams.iter() {
            writeln!(f, "  {}:", name)?;
            for (service, score) in team.scores.iter() {
                writeln!(f, "    {}: {} {}", service, score.up, score.score)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    AlreadyExists,
    DoesNotExist,
    BadValue,
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
                    scores: services
                        .iter()
                        .map(|s| (s.name.to_owned(), Score::default()))
                        .collect(),
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
        config.teams.entry(team_name.to_owned()).and_modify(|team| {
            for (up, service) in team_codes.iter().zip(services.iter()) {
                team.scores
                    .entry(service.name.to_owned())
                    .and_modify(|score| {
                        score.up = *up;
                        if *up {
                            score.score += service.multiplier as u32;
                        }
                        score.history.push_front(*up);
                        if score.history.len() > 10 {
                            score.history.pop_back();
                        }
                    });
            }
        });
    }
}