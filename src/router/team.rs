use axum::{
    extract::{Path, Multipart, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    checker::{passwords::{get_password_groups, overwrite_passwords}, self}, ConfigState,
};

pub fn team_router() -> Router<ConfigState> {
    Router::new()
        .route("/:team/passwords", get(get_team_pw))
        .route("/:team/passwords/:group", post(set_pw))
        .route("/:team/injects/:inject/upload", post(upload_inject_response))
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

async fn upload_inject_response(State(state): State<ConfigState>,Path((team,inject)): Path<(String, String)>, mut multipart: Multipart) -> StatusCode {
    let Ok(Some(field)) = multipart.next_field().await else {
        return StatusCode::BAD_REQUEST;
    };
    let extension = match field.file_name() {
        Some(filename) => filename.split('.')
            .last()
            .unwrap_or("txt")
            .to_string(),
        None => "txt".to_string(),
    };
    let data = field.bytes().await.unwrap();
    let mut config = state.write().await;
    match config.submit_response(&team, &inject, &extension, &data) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::NOT_FOUND,
    }
}
