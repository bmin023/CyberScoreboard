mod admin;
mod team;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::{checker::Score, ConfigState};

pub fn main_router() -> Router<ConfigState> {
    Router::new()
        .nest("/admin", admin::admin_router())
        .nest("/team", team::team_router())
        .route("/scores", get(scores))
        .route("/scores/:team", get(team_scores))
        .route("/time", get(time))
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
