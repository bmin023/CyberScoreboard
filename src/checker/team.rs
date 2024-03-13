use std::{collections::{BTreeMap, VecDeque}, fs};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{injects::InjectResponse, resource_location, Service};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Team {
    pub scores: BTreeMap<String, Score>,
    pub env: Vec<(String, String)>,
    pub inject_responses: Vec<InjectResponse>,
}

impl Team {
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
        self.inject_responses.iter().any(|r| r.inject_uuid == inject_uuid)
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
