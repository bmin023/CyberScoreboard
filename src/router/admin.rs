use std::{sync::{Arc, RwLock}, time::Instant};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    checker::{get_autosave_names, get_save_names, load_save, Config, Score, Service, TeamError},
    password::{get_password_groups, get_passwords, remove_password_group, write_passwords},
};

pub fn admin_router() -> Router<Arc<RwLock<Config>>> {
    Router::new()
        .route("/config", get(admin_info))
        .route(
            "/service/:service",
            post(edit_service).delete(delete_service).get(test_service),
        )
        .route("/service", post(add_service))
        .route("/team/:team/env/:env", post(edit_env).delete(delete_env))
        .route("/team/:team/env", post(add_env))
        .route("/team/:team", post(edit_team).delete(delete_team))
        .route("/team", post(add_team))
        .route("/team/:team/passwords", get(get_team_passwords))
        .route(
            "/team/:team/passwords/:group",
            post(set_passwords).delete(delete_passwords),
        )
        .route("/start", post(start_game))
        .route("/stop", post(stop_game))
        .route("/reset", post(reset_scores))
        .route("/saves", get(get_saves))
}

#[derive(Serialize)]
struct AdminTeam {
    name: String,
    env: Vec<(String, String)>,
}

#[derive(Serialize)]
struct AdminInfo {
    active: bool,
    teams: Vec<AdminTeam>,
    services: Vec<Service>,
}

/// GET the admin config
async fn admin_info(State(state): State<Arc<RwLock<Config>>>) -> Json<AdminInfo> {
    let config = state.read().unwrap();
    let teams = config.teams.iter().map(|(name, team)| AdminTeam {
        name: name.clone(),
        env: team.env.clone(),
    });
    Json(AdminInfo {
        teams: teams.collect(),
        services: config.services.clone(),
        active: config.is_active(),
    })
}

/// POST to edit a service.
/// The Service must not have empty fields and the name must be unique.
async fn edit_service(
    State(state): State<Arc<RwLock<Config>>>,
    Path(service): Path<String>,
    Json(payload): Json<Service>,
) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().unwrap();
    if service != payload.name && config.services.iter().any(|s| s.name == payload.name) {
        return StatusCode::CONFLICT;
    }
    if let Some(service) = config.services.iter_mut().find(|s| s.name == service) {
        *service = payload;
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// DELETE a service.
async fn delete_service(
    State(state): State<Arc<RwLock<Config>>>,
    Path(service): Path<String>,
) -> StatusCode {
    let mut config = state.write().unwrap();
    if let Some(index) = config.services.iter().position(|s| s.name == service) {
        config.services.remove(index);
        for team in config.teams.values_mut() {
            team.scores.remove(index);
        }
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Serialize)]
struct TestResult {
    team: String,
    up: bool,
    message: String,
    error: String,
}

/// GET a test run of a service against all teams
async fn test_service(
    State(state): State<Arc<RwLock<Config>>>,
    Path(service): Path<String>,
) -> Result<Json<Vec<TestResult>>, StatusCode> {
    let config = state.read().unwrap();
    if let Some(service) = config.services.iter().find(|s| s.name == service) {
        let mut results = Vec::new();
        for (name, team) in config.teams.iter() {
            if let Ok(output) = service.check_with_env(&team.env) {
                results.push(TestResult {
                    team: name.clone(),
                    up: output.status.success(),
                    message: String::from_utf8_lossy(&output.stdout).to_string(),
                    error: String::from_utf8_lossy(&output.stderr).to_string(),
                });
            } else {
                results.push(TestResult {
                    team: name.clone(),
                    up: false,
                    message: "Failed to run service".to_string(),
                    error: "Failed to run service".to_string(),
                });
            }
        }
        Ok(Json(results))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// POST to add a service.
/// The Service must not have empty fields and the name must be unique.
async fn add_service(
    State(state): State<Arc<RwLock<Config>>>,
    Json(payload): Json<Service>,
) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().unwrap();
    if config.services.iter().any(|s| s.name == payload.name) {
        StatusCode::CONFLICT
    } else {
        config.services.push(payload);
        for team in config.teams.values_mut() {
            team.scores.push(Score::default());
        }
        StatusCode::OK
    }
}

#[derive(Deserialize, Debug)]
struct EnvPayload {
    name: String,
    value: String,
}

impl EnvPayload {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.value.is_empty()
    }
}

/// POST to edit an environment variable.
/// The Env Variable must not have empty fields and the name must be unique.
async fn edit_env(
    State(state): State<Arc<RwLock<Config>>>,
    Path((team, env)): Path<(String, String)>,
    Json(payload): Json<EnvPayload>,
) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().unwrap();
    if let Some(team) = config.teams.get_mut(&team) {
        if let Some(old_env) = team.env.iter_mut().find(|(name, _)| name == &env) {
            old_env.0 = payload.name;
            old_env.1 = payload.value;
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

/// DELETE an environment variable.
async fn delete_env(
    State(state): State<Arc<RwLock<Config>>>,
    Path((team, env)): Path<(String, String)>,
) -> StatusCode {
    let mut config = state.write().unwrap();
    if let Some(team) = config.teams.get_mut(&team) {
        if let Some(index) = team.env.iter().position(|(name, _)| name == &env) {
            team.env.remove(index);
            StatusCode::OK
        } else {
            StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

/// POST to add an environment variable.
/// The Env Variable must not have empty fields and the name must be unique.
async fn add_env(
    State(state): State<Arc<RwLock<Config>>>,
    Path(team): Path<String>,
    Json(payload): Json<EnvPayload>,
) -> StatusCode {
    print!("Adding env variable: {:?}", payload);
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().unwrap();
    if let Some(team) = config.teams.get_mut(&team) {
        if team.env.iter().any(|(name, _)| name == &payload.name) {
            StatusCode::CONFLICT
        } else {
            team.env.push((payload.name, payload.value));
            StatusCode::OK
        }
    } else {
        StatusCode::NOT_FOUND
    }
}

#[derive(Deserialize)]
struct TeamPayload {
    name: String,
}

/// POST to edit the name of a team
async fn edit_team(
    State(state): State<Arc<RwLock<Config>>>,
    Path(team_name): Path<String>,
    Json(payload): Json<TeamPayload>,
) -> StatusCode {
    let mut config = state.write().unwrap();
    if config.teams.contains_key(&payload.name) {
        return StatusCode::CONFLICT;
    }
    if let Some(team) = config.teams.remove(&team_name) {
        config.teams.insert(payload.name, team);
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// DELETE a team
async fn delete_team(
    State(state): State<Arc<RwLock<Config>>>,
    Path(team): Path<String>,
) -> StatusCode {
    let mut config = state.write().unwrap();
    if config.teams.remove(&team).is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// POST to add a team
async fn add_team(
    State(state): State<Arc<RwLock<Config>>>,
    Json(payload): Json<TeamPayload>,
) -> StatusCode {
    let mut config = state.write().unwrap();
    match config.add_team(payload.name) {
        Ok(_) => StatusCode::OK,
        Err(TeamError::AlreadyExists) => StatusCode::CONFLICT,
        Err(TeamError::InvalidName) => StatusCode::BAD_REQUEST,
    }
}

async fn stop_game(State(state): State<Arc<RwLock<Config>>>) -> StatusCode {
    let mut config = state.write().unwrap();
    config.stop();
    StatusCode::OK
}

async fn start_game(State(state): State<Arc<RwLock<Config>>>) -> StatusCode {
    let mut config = state.write().unwrap();
    config.start();
    StatusCode::OK
}

async fn reset_scores(State(state): State<Arc<RwLock<Config>>>) -> StatusCode {
    let mut config = state.write().unwrap();
    config.reset_scores();
    StatusCode::OK
}

#[derive(Serialize)]
struct PasswordBody {
    group: String,
    passwords: String,
}

async fn get_team_passwords(
    Path(team): Path<String>,
) -> Result<Json<Vec<PasswordBody>>, StatusCode> {
    let Ok(groups) = get_password_groups(&team) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let body = groups
        .iter()
        .filter_map(|group| match get_passwords(&team, group) {
            Ok(passwords) => Some(PasswordBody {
                group: group.clone(),
                passwords: passwords.clone(),
            }),
            Err(_) => None,
        })
        .collect();
    Ok(Json(body))
}

#[derive(Deserialize)]
struct PasswordPayload {
    passwords: String,
}

async fn set_passwords(
    Path((team, group)): Path<(String, String)>,
    Json(payload): Json<PasswordPayload>,
) -> StatusCode {
    match write_passwords(&team, &group, &payload.passwords) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

async fn delete_passwords(Path((team, group)): Path<(String, String)>) -> StatusCode {
    match remove_password_group(&team, &group) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[derive(Serialize)]
struct SaveBody {
    name: String,
    #[serde(with = "serde_millis")]
    timestamp: Instant,
}

#[derive(Serialize)]
struct SavesWrapper {
    saves: Vec<SaveBody>,
    autosaves: Vec<SaveBody>,
}

async fn get_saves() -> Json<SavesWrapper> {
    let saves = get_save_names()
        .iter()
        .map(|name| {
            let timestamp = if let Ok(save) = load_save(name) {
                save.saved_at
            } else {
                Instant::now()
            };
            SaveBody {
                name: name.clone(),
                timestamp,
            }
        })
        .collect();
    let autosaves = get_autosave_names()
        .iter()
        .map(|name| {
            let name = "autosave/".to_owned() + name;
            let timestamp = if let Ok(save) = load_save(&name) {
                save.saved_at
            } else {
                Instant::now()
            };
            SaveBody { name, timestamp }
        })
        .collect();
    Json(SavesWrapper { saves, autosaves })
}
