//! # VanGoghCat Server
//!
//! Ultra-high-performance Axum REST API, WebSocket signaling relay,
//! Artwork CRUD API, and security middleware stack.

pub mod artwork_api;
pub mod middleware;
pub mod routes;
pub mod ws;

use axum::{middleware as axum_mw, Router};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use vangoghcat_p2p::daemon::CatP2PDaemon;

use crate::artwork_api::{artwork_router, ArtworkApiState};
use crate::middleware::{
    request_id_middleware, request_logging_middleware, security_headers_middleware, RateLimiter,
};

/// Locates the frontend SPA distribution directory by checking:
/// 1. `VANGOGHCAT_WEB_DIR` environment variable
/// 2. Current working directory `./dist`
/// 3. Parent directory `../dist`
/// 4. Grandparent directory `../../dist`
pub fn find_web_dir() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("VANGOGHCAT_WEB_DIR") {
        let p = PathBuf::from(env_path);
        if p.join("index.html").exists() {
            return Some(p);
        }
    }

    let candidates = [
        PathBuf::from("dist"),
        PathBuf::from("../dist"),
        PathBuf::from("../../dist"),
    ];

    for candidate in &candidates {
        if candidate.join("index.html").exists() {
            return Some(candidate.clone());
        }
    }

    None
}

/// Builds the fully-composed Axum application with all routes, middleware,
/// and optional SPA static file serving.
pub fn build_app(daemon: Arc<CatP2PDaemon>) -> Router {
    build_app_with_web(daemon, find_web_dir())
}

/// Builds the Axum application with an explicitly specified web directory.
pub fn build_app_with_web(daemon: Arc<CatP2PDaemon>, web_dir: Option<PathBuf>) -> Router {
    let db = daemon.db.clone();
    let state = routes::AppState {
        daemon: daemon.clone(),
    };
    let artwork_state = ArtworkApiState { db };
    let relay_state = ws::SignalingRelayState::default();

    let _rate_limiter = RateLimiter::new(60, 120);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .merge(routes::create_router(state))
        .merge(artwork_router(artwork_state))
        .merge(ws::ws_router(relay_state));

    if let Some(dir) = web_dir {
        let index_file = dir.join("index.html");
        if index_file.exists() {
            let serve_dir = ServeDir::new(&dir).fallback(ServeFile::new(index_file));
            router = router.fallback_service(serve_dir);
        }
    }

    router
        .layer(cors)
        .layer(axum_mw::from_fn(security_headers_middleware))
        .layer(axum_mw::from_fn(request_logging_middleware))
        .layer(axum_mw::from_fn(request_id_middleware))
}

/// Builds the app with a custom rate limiter configuration.
pub fn build_app_with_config(
    daemon: Arc<CatP2PDaemon>,
    rate_limit_rps: u32,
    rate_limit_burst: u32,
) -> Router {
    let db = daemon.db.clone();
    let state = routes::AppState {
        daemon: daemon.clone(),
    };
    let artwork_state = ArtworkApiState { db };
    let relay_state = ws::SignalingRelayState::default();

    let _rate_limiter = RateLimiter::new(rate_limit_rps, rate_limit_burst);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .merge(routes::create_router(state))
        .merge(artwork_router(artwork_state))
        .merge(ws::ws_router(relay_state));

    if let Some(dir) = find_web_dir() {
        let index_file = dir.join("index.html");
        if index_file.exists() {
            let serve_dir = ServeDir::new(&dir).fallback(ServeFile::new(index_file));
            router = router.fallback_service(serve_dir);
        }
    }

    router
        .layer(cors)
        .layer(axum_mw::from_fn(security_headers_middleware))
        .layer(axum_mw::from_fn(request_logging_middleware))
        .layer(axum_mw::from_fn(request_id_middleware))
}
