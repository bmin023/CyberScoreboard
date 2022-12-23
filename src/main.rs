mod checker;
mod router;

use axum::Router;
use axum_extra::routing::SpaRouter;
use checker::Config;
use std::{net::SocketAddr, sync::{Arc, RwLock}, time::Duration, thread};
use tokio::time;
use tower_http::cors::{Any, CorsLayer};
use tracing::{Level, info_span, info, error};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    // console_subscriber::init();
    let state = Arc::new(RwLock::new(Config::new()));
    let score_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            let another_clone = Arc::clone(&score_state);
            interval.tick().await;
            thread::spawn(move || {
                let thread_arc = Arc::clone(&another_clone);
                let span = info_span!("Game Loop");
                let _enter = span.enter();
                let mut config = {
                    thread_arc.read().unwrap().clone()
                };
                info!("Game Tick: {}",config.run_time().as_secs());
                {
                    config.score_tick();
                    let mut truth = thread_arc.write().unwrap();
                    truth.smart_combine(config);
                };
            });
        }
    });
    let save_loop_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(600));
        loop {
            interval.tick().await;
            let config = save_loop_state.read().unwrap();
            if let Err(err) = config.autosave() {
                error!("Failed to autosave: {:?}", err);
            }
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let app = Router::new()
        .nest("/api", router::main_router())
        .merge(SpaRouter::new("/assets", "public/assets").index_file("../index.html"))
        .layer(cors)
        .with_state(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
