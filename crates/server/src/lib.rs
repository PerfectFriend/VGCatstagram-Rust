//! # VanGoghCat Server
//!
//! Ultra-high-performance Axum REST API, WebSocket signaling relay,
//! Artwork CRUD API, and security middleware stack.

pub mod artwork_api;
pub mod middleware;
pub mod routes;
pub mod ws;

use axum::{middleware as axum_mw, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use vangoghcat_p2p::daemon::CatP2PDaemon;

use crate::artwork_api::{artwork_router, ArtworkApiState};
use crate::middleware::{
    request_id_middleware, request_logging_middleware, security_headers_middleware, RateLimiter,
};

/// Builds the fully-composed Axum application with all routes and middleware.
///
/// # Middleware Stack (applied bottom-to-top):
/// 1. Request ID injection (`X-Request-Id`)
/// 2. Structured request logging (method, path, status, latency)
/// 3. Security headers (nosniff, frame deny, XSS protection)
/// 4. CORS (configurable, defaults to permissive)
/// 5. Rate limiting (token bucket per IP)
pub fn build_app(daemon: Arc<CatP2PDaemon>) -> Router {
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

    Router::new()
        .merge(routes::create_router(state))
        .merge(artwork_router(artwork_state))
        .merge(ws::ws_router(relay_state))
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

    Router::new()
        .merge(routes::create_router(state))
        .merge(artwork_router(artwork_state))
        .merge(ws::ws_router(relay_state))
        .layer(cors)
        .layer(axum_mw::from_fn(security_headers_middleware))
        .layer(axum_mw::from_fn(request_logging_middleware))
        .layer(axum_mw::from_fn(request_id_middleware))
}
