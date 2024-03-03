use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    checker::{passwords, saves, injects, Config, config::ConfigError, Service, TeamError},
    checker::injects::InjectUser,
    ConfigState,
};

pub fn admin_router() -> Router<ConfigState> {
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
        .route("/saves", get(get_saves).post(save))
        .route("/saves/load", post(load_save))
        .route("/injects", get(get_injects).post(add_inject))
        .route("/injects/:inject_uuid", post(edit_inject).delete(delete_inject))
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
async fn admin_info(State(state): State<ConfigState>) -> Json<AdminInfo> {
    let config = state.read().await;
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
    State(state): State<ConfigState>,
    Path(service): Path<String>,
    Json(payload): Json<Service>,
) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().await;
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
    State(state): State<ConfigState>,
    Path(service): Path<String>,
) -> StatusCode {
    let mut config = state.write().await;
    match config.remove_service(&service) {
        Ok(_) => StatusCode::OK,
        Err(ConfigError::DoesNotExist) => StatusCode::NOT_FOUND,
        Err(ConfigError::AlreadyExists) => StatusCode::CONFLICT,
        Err(ConfigError::BadValue) => StatusCode::BAD_REQUEST,
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
    State(state): State<ConfigState>,
    Path(service): Path<String>,
) -> Result<Json<Vec<TestResult>>, StatusCode> {
    let config = state.read().await;
    if let Some(service) = config.services.iter().find(|s| s.name == service) {
        let mut results = Vec::new();
        for (name, team) in config.teams.iter() {
            if let Ok(output) = service.check_with_env(&team.env).await {
                results.push(TestResult {
                    team: name.clone(),
                    up: output.up,
                    message: output.message,
                    error: output.error,
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
async fn add_service(State(state): State<ConfigState>, Json(payload): Json<Service>) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().await;
    match config.add_service(payload) {
        Ok(_) => StatusCode::OK,
        Err(ConfigError::AlreadyExists) => StatusCode::CONFLICT,
        Err(ConfigError::BadValue) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
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
    State(state): State<ConfigState>,
    Path((team, env)): Path<(String, String)>,
    Json(payload): Json<EnvPayload>,
) -> StatusCode {
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().await;
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
    State(state): State<ConfigState>,
    Path((team, env)): Path<(String, String)>,
) -> StatusCode {
    let mut config = state.write().await;
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
    State(state): State<ConfigState>,
    Path(team): Path<String>,
    Json(payload): Json<EnvPayload>,
) -> StatusCode {
    print!("Adding env variable: {:?}", payload);
    if !payload.is_valid() {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().await;
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
    State(state): State<ConfigState>,
    Path(team_name): Path<String>,
    Json(payload): Json<TeamPayload>,
) -> StatusCode {
    let mut config = state.write().await;
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
async fn delete_team(State(state): State<ConfigState>, Path(team): Path<String>) -> StatusCode {
    let mut config = state.write().await;
    if config.teams.remove(&team).is_some() {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// POST to add a team
async fn add_team(
    State(state): State<ConfigState>,
    Json(payload): Json<TeamPayload>,
) -> StatusCode {
    let mut config = state.write().await;
    match config.add_team(payload.name) {
        Ok(_) => StatusCode::OK,
        Err(TeamError::AlreadyExists) => StatusCode::CONFLICT,
        Err(TeamError::InvalidName) => StatusCode::BAD_REQUEST,
    }
}

async fn stop_game(State(state): State<ConfigState>) -> StatusCode {
    let mut config = state.write().await;
    config.stop();
    StatusCode::OK
}

async fn start_game(State(state): State<ConfigState>) -> StatusCode {
    let mut config = state.write().await;
    config.start();
    StatusCode::OK
}

async fn reset_scores(State(state): State<ConfigState>) -> StatusCode {
    let mut config = state.write().await;
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
    let Ok(groups) = passwords::get_password_groups(&team) else {
        return Err(StatusCode::NOT_FOUND);
    };
    let body = groups
        .iter()
        .filter_map(|group| match passwords::get_passwords(&team, group) {
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
    match passwords::write_passwords(&team, &group, &payload.passwords) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

async fn delete_passwords(Path((team, group)): Path<(String, String)>) -> StatusCode {
    match passwords::remove_password_group(&team, &group) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[derive(Serialize)]
struct SaveBody {
    name: String,
    timestamp: u128,
}

#[derive(Serialize)]
struct SavesWrapper {
    saves: Vec<SaveBody>,
    autosaves: Vec<SaveBody>,
}

#[tracing::instrument]
async fn get_saves() -> Json<SavesWrapper> {
    let saves = saves::get_save_names()
        .iter()
        .map(|name| {
            let timestamp = if let Ok(save) = saves::load_save(name) {
                save.saved_at
            } else {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            };
            SaveBody {
                name: name.clone(),
                timestamp,
            }
        })
        .collect();
    let autosaves = saves::get_autosave_names()
        .iter()
        .map(|name| {
            let name = "autosave/".to_owned() + name;
            let timestamp = if let Ok(save) = saves::load_save(&name) {
                save.saved_at
            } else {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            };
            SaveBody { name, timestamp }
        })
        .collect();
    Json(SavesWrapper { saves, autosaves })
}

#[derive(Deserialize)]
struct SavePayload {
    name: String,
}

async fn save(State(state): State<ConfigState>, Json(payload): Json<SavePayload>) -> StatusCode {
    let config = state.read().await;
    match config.save(payload.name.as_str()) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn load_save(
    State(state): State<ConfigState>,
    Json(payload): Json<SavePayload>,
) -> StatusCode {
    let mut old_config = state.write().await;
    let Ok(config) = Config::from_save(&payload.name) else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    *old_config = config;
    StatusCode::OK
}

async fn get_injects(
    State(state): State<ConfigState>,
) -> Result<Json<Vec<injects::Inject>>, StatusCode> {
    let config = state.read().await;
    Ok(Json(config.injects.clone()))
}

async fn add_inject(
    State(state): State<ConfigState>,
    Json(payload): Json<injects::CreateInject>,
) -> StatusCode {
    let mut config = state.write().await;
    config.add_inject(payload);
    StatusCode::OK
}

async fn edit_inject(
    State(state): State<ConfigState>,
    Path(inject_uuid): Path<Uuid>,
    Json(payload): Json<injects::Inject>
) -> StatusCode {
    if payload.uuid != inject_uuid {
        return StatusCode::BAD_REQUEST;
    }
    let mut config = state.write().await;
    match config.edit_inject(payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

async fn delete_inject(
    State(state): State<ConfigState>,
    Path(inject_uuid): Path<Uuid>,
) -> StatusCode {
    let mut config = state.write().await;
    match config.delete_inject(inject_uuid) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

