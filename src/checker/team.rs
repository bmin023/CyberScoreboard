use std::{
    collections::{BTreeMap, VecDeque},
    fs,
};

use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{injects::InjectResponse, resource_location, Service};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Team {
    pub name: String,
    pub id: Uuid,
    pub scores: BTreeMap<String, Score>,
    pub env: Vec<(String, String)>,
    pub inject_responses: Vec<InjectResponse>,
}

impl Team {
    pub fn from_services(services: &Vec<Service>) -> Self {
        Self {
            name: String::new(),
            id: Uuid::new_v4(),
            scores: services
                .iter()
                .map(|s| (s.name.to_owned(), Score::default()))
                .collect(),
            env: vec![],
            inject_responses: vec![],
        }
    }
    pub fn score(&self) -> u32 {
        self.scores.iter().map(|(_, s)| s.score).sum()
    }
    pub fn get_reponses(&self, inject_uuid: Uuid) -> Vec<InjectResponse> {
        self.inject_responses
            .iter()
            .filter(|r| r.inject_uuid == inject_uuid)
            .cloned()
            .collect()
    }
    pub fn has_response(&self, inject_uuid: Uuid) -> bool {
        self.inject_responses
            .iter()
            .any(|r| r.inject_uuid == inject_uuid)
    }
    pub fn has_passwd(&self) -> bool {
        self.env.iter().find(|(k,_)| k == "TEAM_PASSWORD").is_some()
    }
    pub fn check_passwd(&self, password: &String) -> bool {
        self.env
            .iter()
            .find(|(k, v)| k == "TEAM_PASSWORD" && v == password)
            .is_some()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

pub enum TeamError {
    InvalidName,
    AlreadyExists,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Score {
    pub score: u32,
    pub up: bool,
    pub history: VecDeque<bool>,
}

pub fn load_teams(services: &Vec<Service>) -> BTreeMap<String, Team> {
    let team_file = std::env::var("SB_TEAMS").unwrap_or_else(|_| "teams.yaml".to_string());
    let file = fs::read_to_string(format!("{}/{}", resource_location(), team_file))
        .expect(format!("{} should be in the resource directory", team_file).as_str());
    let teams = serde_yaml::from_str::<BTreeMap<String, BTreeMap<String, String>>>(&file)
        .expect(format!("{} should be formatted correctly", team_file).as_str());
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
                    name: name.clone(),
                    id: Uuid::new_v4(),
                    scores: services
                        .iter()
                        .map(|s| (s.name.to_owned(), Score::default()))
                        .collect(),
                    env,
                    inject_responses: vec![],
                },
            )
        })
        .collect::<BTreeMap<String, Team>>();
    teams
}

impl AuthUser for Team {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.id.as_bytes()
    }
}
