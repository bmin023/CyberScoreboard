use std::collections::HashMap;

use axum::{
    extract::{Multipart, Path, Request, State},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    checker::{
        injects::{Inject, InjectResponse, InjectUser},
        passwords::{get_password_groups, overwrite_passwords}, Score,
    },
    ConfigState,
};

use super::AuthSession;

async fn check_if_team(
    Path(path): Path<HashMap<String, String>>,
    State(state): State<ConfigState>,
    mut auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(team_name) = path.get("team") else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let name_matches = if let Some(user) = &auth.user {
        &user.1.clone() == team_name || user.is_admin()
    } else {
        false
    };
    if !name_matches {
        let config = state.read().await;
        let Some(team) = config.teams.get(team_name) else {
            return Err(StatusCode::NOT_FOUND);
        };
        if team.has_passwd() {
            return Err(StatusCode::UNAUTHORIZED);
        }
        if auth.user.is_none() && auth.login(&team.into()).await.is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    let response = next.run(request).await;
    Ok(response)
}

pub fn team_router(state: ConfigState) -> Router<ConfigState> {
    Router::new()
        .route("/:team/passwords", get(get_team_pw))
        .route("/:team/passwords/:group", post(set_pw))
        .route(
            "/:team/injects/:inject_uuid/upload",
            post(upload_inject_response),
        )
        .route("/:team/injects", get(get_injects))
        .route("/:team/injects/:inject_uuid", get(get_inject))
        .route("/:team/scores", get(team_scores))
        .layer(middleware::from_fn_with_state(state, check_if_team))
}

#[derive(Serialize)]
struct TeamScores {
    services: Vec<String>,
    scores: Vec<Score>,
}

async fn team_scores(
    State(state): State<ConfigState>,
    Path(team): Path<String>,
) -> Result<Json<TeamScores>, StatusCode> {
    let config = state.read().await;
    if let Some(team) = config.teams.get(&team) {
        let team_scores = config.services.iter().fold(
            TeamScores {
                services: Vec::new(),
                scores: Vec::new(),
            },
            |mut acc, s| {
                acc.services.push(s.name.clone());
                acc.scores.push(
                    team.scores
                        .get(&s.name)
                        .unwrap_or(&Score::default())
                        .clone(),
                );
                acc
            },
        );
        Ok(Json(team_scores))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_team_pw(Path(team): Path<String>) -> Result<Json<Vec<String>>, StatusCode> {
    if let Ok(groups) = get_password_groups(&team) {
        Ok(Json(groups))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
struct PasswordPayload {
    passwords: String,
}

async fn set_pw(
    Path((team, group)): Path<(String, String)>,
    Json(payload): Json<PasswordPayload>,
) -> StatusCode {
    match overwrite_passwords(&team, &group, &payload.passwords) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[tracing::instrument(skip(state, team, inject_uuid, multipart))]
async fn upload_inject_response(
    State(state): State<ConfigState>,
    Path((team, inject_uuid)): Path<(String, Uuid)>,
    mut multipart: Multipart,
) -> StatusCode {
    let Ok(Some(field)) = multipart.next_field().await else {
        return StatusCode::BAD_REQUEST;
    };
    let filename = match field.file_name() {
        Some(filename) => filename.to_string(),
        None => "unknown".to_string(),
    };
    // handle unwrap
    let data = field.bytes().await;
    if data.is_err() {
        error!(
            "{} submitted file too large for inject {}",
            team, inject_uuid
        );
        return StatusCode::PAYLOAD_TOO_LARGE;
    }
    let mut config = state.write().await;
    if let Ok(bytes) = data {
        match config.submit_response(&team, inject_uuid, &filename, &bytes) {
            Ok(_) => {
                info!("{} submitted response for inject {}", team, inject_uuid);
                StatusCode::OK
            }
            Err(_) => StatusCode::NOT_FOUND,
        }
    } else {
        error!("Error when decoding multipart response from team {}", team);
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Serialize)]
struct InjectRequest {
    active_injects: Vec<InjectDesc>,
    completed_injects: Vec<InjectResponse>,
}

#[derive(Serialize)]
struct InjectDesc {
    uuid: Uuid,
    name: String,
    start: u32,
    duration: u32,
    completed: bool,
    file_type: Option<Vec<String>>,
    sticky: bool,
}

impl InjectDesc {
    fn from_inject(inject: Inject) -> Self {
        InjectDesc {
            uuid: inject.uuid,
            name: inject.name,
            start: inject.start,
            duration: inject.duration,
            completed: inject.completed,
            file_type: inject.file_type,
            sticky: inject.sticky,
        }
    }
}

async fn get_injects(
    State(state): State<ConfigState>,
    Path(team): Path<String>,
) -> Result<Json<InjectRequest>, StatusCode> {
    let config = state.read().await;
    let completed_injects = config
        .teams
        .get(&team)
        .ok_or(StatusCode::NOT_FOUND)?
        .inject_responses
        .clone();
    let active_injects = config
        .get_injects_for_team(&team)
        .ok()
        .ok_or(StatusCode::NOT_FOUND)?
        .iter()
        .cloned()
        .map(|inject| InjectDesc::from_inject(inject))
        .collect();
    Ok(Json(InjectRequest {
        active_injects,
        completed_injects,
    }))
}

#[derive(Serialize)]
struct InjectData {
    desc: InjectDesc,
    html: String,
    history: Vec<InjectResponse>,
}

async fn get_inject(
    State(state): State<ConfigState>,
    Path((team, inject_uuid)): Path<(String, Uuid)>,
) -> Result<Json<InjectData>, StatusCode> {
    let config = state.read().await;
    let inject = config
        .get_inject(inject_uuid)
        .ok_or(StatusCode::NOT_FOUND)?;
    let team = config.teams.get(&team).ok_or(StatusCode::NOT_FOUND)?;
    let html = inject.get_html(&team.env);
    let history = team.get_reponses(inject_uuid);
    Ok(Json(InjectData {
        desc: InjectDesc::from_inject(inject),
        html,
        history,
    }))
}
