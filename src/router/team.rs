use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    checker::passwords::{get_password_groups, overwrite_passwords}, ConfigState,
};

pub fn team_router() -> Router<ConfigState> {
    Router::new()
        .route("/:team/passwords", get(get_team_pw))
        .route("/:team/passwords/:group", post(set_pw))
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
