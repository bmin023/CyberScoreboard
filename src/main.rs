mod checker;
mod router;
mod password;

use axum::Router;
use axum_extra::routing::SpaRouter;
use checker::Config;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time;
use tower_http::cors::{Any, CorsLayer};

use crate::checker::autosave;

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(Config::new()));
    let score_loop_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            let mut config = score_loop_state.write().unwrap();
            println!("-- Game Time: {} --", config.run_time().as_secs());
            config.score_tick();
        }
    });
    let save_loop_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let config = save_loop_state.read().unwrap();
            if let Err(_) = autosave(&config) {
                println!("Wasn't able to autosave.");
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