use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    checker::{
        injects::{Inject, InjectResponse},
        passwords::{get_password_groups, overwrite_passwords},
    },
    ConfigState,
};

pub fn team_router() -> Router<ConfigState> {
    Router::new()
        .route("/:team/passwords", get(get_team_pw))
        .route("/:team/passwords/:group", post(set_pw))
        .route(
            "/:team/injects/:inject_uuid/upload",
            post(upload_inject_response),
        )
        .route("/:team/injects", get(get_injects))
        .route("/:team/injects/:inject_uuid", get(get_inject))
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
    let data = field.bytes().await.unwrap();
    let mut config = state.write().await;
    match config.submit_response(&team, inject_uuid, &filename, &data) {
        Ok(_) => {
            info!("{} submitted response for inject {}", team, inject_uuid);
            StatusCode::OK
        },
        Err(_) => StatusCode::NOT_FOUND,
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
    let history = team
        .get_reponses(inject_uuid);
    Ok(Json(InjectData {
        desc: InjectDesc::from_inject(inject),
        html,
        history,
    }))
}
