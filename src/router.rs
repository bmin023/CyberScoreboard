mod admin;
mod team;

use std::sync::{Arc, RwLock};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::checker::{Config, Score};

pub fn main_router() -> Router<Arc<RwLock<Config>>> {
    Router::new()
        .nest("/admin", admin::admin_router())
        .nest("/team", team::team_router())
        .route("/scores", get(scores))
        .route("/scores/:team", get(team_scores))
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

async fn scores(State(state): State<Arc<RwLock<Config>>>) -> Json<ScoreWrapper> {
    let config = state.read().unwrap();
    let scores = config.teams.iter().map(|(name, team)| ScoreBody {
        name: name.clone(),
        score: team.score(),
        ups: team.scores.iter().map(|s| s.up).collect(),
    });
    let services = config.services.iter().map(|s| s.name.clone());
    Json(ScoreWrapper {
        teams: scores.collect(),
        services: services.collect(),
    })
}

#[derive(Serialize)]
struct TeamScore {
    services: Vec<String>,
    scores: Vec<Score>,
}

async fn team_scores(
    State(state): State<Arc<RwLock<Config>>>,
    Path(team): Path<String>,
) -> Result<Json<TeamScore>, StatusCode> {
    let config = state.read().unwrap();
    if let Some(team) = config.teams.get(&team) {
        let services = config.services.iter().map(|s| s.name.clone());
        Ok(Json(TeamScore {
            services: services.collect(),
            scores: team.scores.clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
