mod auth;
mod checker;
mod router;

use axum::http::StatusCode;
use axum::routing::get_service;
use axum::Router;
use checker::injects::InjectUser;
use checker::Config;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::{debug, debug_span, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::checker::resource_location;

pub type ConfigState = Arc<RwLock<Config>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_env("LOG_LEVEL")
                .unwrap_or_else(|_| "scoreboard=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let version = env!("CARGO_PKG_VERSION");
    info!("Starting Scoreboard v{}", version);
    let state = Arc::new(RwLock::new(Config::new()));
    let score_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            debug!("Game Tick");
            let mut config = { score_state.read().await.clone() };
            if config.is_active() {
                config.inject_tick();
                config.score_tick().await;
                let mut truth = score_state.write().await;
                truth.inject_tick();
                truth.smart_combine(config);
            }
            debug!("Game Tick Complete");
        }
    });
    let save_loop_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(600));
        loop {
            interval.tick().await;
            let span = debug_span!("Save Loop");
            let _enter = span.enter();
            let config = save_loop_state.read().await;
            debug!("Autosaving");
            if let Err(err) = config.autosave() {
                error!("Failed to autosave: {:?}", err);
            }
        }
    });
    let download_dir = ServeDir::new(format!("{}/downloads", resource_location()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", router::main_router(state.clone()))
        .nest_service("/downloads", download_dir)
        .nest_service(
            "/assets",
            get_service(ServeDir::new("./public/assets")
            ),
        )
        .fallback_service(
            get_service(ServeFile::new("./public/index.html")).handle_error(
                |_| async move { (StatusCode::INTERNAL_SERVER_ERROR, "internal server error") },
            ),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(cors)
        .with_state(state);
    let port = std::env::var("SB_PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse::<u16>().expect("Invalid Port")));

    info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    let _ = axum::serve(listener, app).await.unwrap();
}
