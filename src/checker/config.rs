use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::error;

use std::fmt::Display;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

use super::inject::load_injects;
use super::injects::Inject;
use super::password::{load_password_saves, validate_password_fs};
use super::save::{autosave, load_save, save_config, validate_save_fs, SaveError};
use super::service::load_services;
use super::team::load_teams;
use super::{Score, Service, Team, TeamError};

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
            let var_name = Err(TeamError::AlreadyExists);
            return var_name;
        }
        self.teams.insert(
            name.clone(),
            Team::from_services(&self.services),
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
    pub async fn score_tick(&mut self) {
        score_teams(self).await;
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

    pub fn get_team_with_password(&self,team: &String, password: &String) -> Option<&Team> {
        self.teams.get(team).map_or_else(|| None, |v| if v.check_passwd(password) { Some(v) } else { None })
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
            if let Some(index) = self.injects.iter().position(|i| i.uuid == inject.uuid) {
                if (inject.completed && !self.injects[index].completed)
                    && self.injects[index].is_ended((self.run_time().as_secs() / 60) as u32)
                {
                    self.injects[index].completed = true;
                }
            } else {
                error!(
                    "Couldn't resolve inject: {}. It was probably removed during a score tick.",
                    inject.name
                );
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

async fn score_teams(config: &mut Config) {
    let services = &config.services;
    let mut set = JoinSet::new();

    for (name, team) in &config.teams {
        for check in services.iter().cloned() {
            let env = team.env.clone();
            let name = name.clone();
            set.spawn(async move {
                let Ok(output) = check.check_with_env(&env).await else {
                    return None;
                };
                Some((name, check.name, output.up))
            });
        }
    }

    while let Some(res) = set.join_next().await {
        let Ok(res) = res else {
            continue;
        };
        if let Some((team_name, service_name, up)) = res {
            config.teams.entry(team_name).and_modify(|team| {
                team.scores.entry(service_name).and_modify(|score| {
                    score.up = up;
                    if up {
                        score.score += 1;
                    }
                    score.history.push_front(up);
                    if score.history.len() > 10 {
                        score.history.pop_back();
                    }
                });
            });
        }
    }
}
