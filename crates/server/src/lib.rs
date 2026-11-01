//! # VanGoghCat Server
//!
//! Ultra-high-performance Axum REST API and WebSocket signaling server.

pub mod routes;
pub mod ws;

use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use vangoghcat_p2p::daemon::CatP2PDaemon;

pub fn build_app(daemon: Arc<CatP2PDaemon>) -> Router {
    let state = routes::AppState { daemon };
    let relay_state = ws::SignalingRelayState::default();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(routes::create_router(state))
        .merge(ws::ws_router(relay_state))
        .layer(cors)
}
