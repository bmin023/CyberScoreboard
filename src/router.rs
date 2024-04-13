mod admin;
mod team;

use axum::{
    extract::{Path, State}, http::StatusCode, response::{IntoResponse, Redirect}, routing::{get, post}, Json, Router
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{auth::{Auth, TeamCredentials}, checker::ScoreboardInfo};

use axum_login::{
    tower_sessions::{MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};

use crate::{checker::Score, ConfigState};

pub type AuthSession = axum_login::AuthSession<Auth>;

pub fn main_router(state: ConfigState) -> Router<ConfigState> {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    let backend = Auth::new(&state);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    Router::new()
        .nest("/admin", admin::admin_router())
        .nest("/team", team::team_router(state))
        .route("/scores", get(scores))
        .route("/time", get(time))
        .route("/login", post(login))
        .route("/info", get(scoreboard_info))
        .layer(auth_layer)
}

#[derive(Serialize)]
struct ScoreWrapper {
    teams: Vec<ScoreBody>,
    services: Vec<String>,
}

#[derive(Serialize)]
struct ScoreBody {
    name: String,
    score: u32,
    ups: Vec<bool>,
}

#[derive(Serialize)]
struct TimeBody {
    minutes: u64,
    seconds: u64,
    active: bool,
}

async fn time(State(state): State<ConfigState>) -> Json<TimeBody> {
    let config = state.read().await;
    let runtime = config.run_time();
    Json(TimeBody {
        minutes: runtime.as_secs() / 60,
        seconds: runtime.as_secs() % 60,
        active: config.is_active(),
    })
}

async fn scoreboard_info() -> Json<ScoreboardInfo> {
    Json(ScoreboardInfo::default())
}

async fn scores(State(state): State<ConfigState>) -> Json<ScoreWrapper> {
    let config = state.read().await;
    let services = config.services.iter().map(|s| s.name.clone());
    let scores = config.teams.iter().map(|(name, team)| ScoreBody {
        name: name.to_owned(),
        score: team.score(),
        ups: config
            .services
            .iter()
            .map(|s| team.scores.get(&s.name).unwrap_or(&Score::default()).up)
            .collect(),
    });
    Json(ScoreWrapper {
        teams: scores.collect(),
        services: services.collect(),
    })
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn login(
    mut auth: AuthSession,
    Json(payload): Json<LoginPayload>,
) -> Result<StatusCode, StatusCode> {
    let creds = TeamCredentials {
        name: payload.username,
        password: payload.password,
    };
    if let Ok(Some(user)) = auth.authenticate(creds).await {
        auth.login(&user).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
