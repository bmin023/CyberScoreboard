mod admin;
mod team;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::{ConfigState, checker::Score};

pub fn main_router() -> Router<ConfigState> {
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

#[derive(Serialize)]
struct TeamScore {
    services: Vec<String>,
    scores: Vec<Score>,
}

async fn team_scores(
    State(state): State<ConfigState>,
    Path(team): Path<String>,
) -> Result<Json<TeamScore>, StatusCode> {
    let config = state.read().await;
    if let Some(team) = config.teams.get(&team) {
        let services = config.services.iter().map(|s| s.name.clone());
        let scores = team
            .scores
            .iter()
            .filter(|(name, _)| config.services.iter().any(|s| &s.name == *name))
            .map(|(_, score)| score.clone());
        Ok(Json(TeamScore {
            services: services.collect(),
            scores: scores.collect(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
