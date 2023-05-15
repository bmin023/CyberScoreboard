use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

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
            "/:team/injects/:inject/upload",
            post(upload_inject_response),
        )
        .route("/:team/injects", get(get_injects))
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
    Path((team, inject)): Path<(String, String)>,
    mut multipart: Multipart,
) -> StatusCode {
    let Ok(Some(field)) = multipart.next_field().await else {
        return StatusCode::BAD_REQUEST;
    };
    let extension = match field.file_name() {
        Some(filename) => filename.split('.').last().unwrap_or("txt").to_string(),
        None => "txt".to_string(),
    };
    let data = field.bytes().await.unwrap();
    let mut config = state.write().await;
    match config.submit_response(&team, &inject, &extension, &data) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[derive(Serialize)]
struct InjectRequest {
    active_injects: Vec<Inject>,
    completed_injects: Vec<InjectResponse>,
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
        .expect("This check should already have been done above");
    Ok(Json(InjectRequest {
        active_injects,
        completed_injects,
    }))
}
