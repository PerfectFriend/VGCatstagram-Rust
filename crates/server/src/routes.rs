//! Axum REST route handlers for VanGoghCat server.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vangoghcat_core::types::StorageContractReq;
use vangoghcat_p2p::{
    contract::handle_contract_request,
    daemon::{CatP2PDaemon, DaemonStatus},
    smp::SmpMessage,
    xftp::{XftpEngine, XftpManifest},
};

#[derive(Clone)]
pub struct AppState {
    pub daemon: Arc<CatP2PDaemon>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

#[derive(Deserialize)]
pub struct SendSmpReq {
    pub queue_id: String,
    pub sender_node_id: String,
    pub payload_encrypted_hex: String,
    pub ttl_seconds: Option<u32>,
}

#[derive(Deserialize)]
pub struct PackageArtReq {
    pub raw_base64: String,
    pub chunk_size: Option<usize>,
}

#[derive(Serialize)]
pub struct PackageArtResp {
    pub manifest: XftpManifest,
    pub chunks_count: usize,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/catp2p/status", get(catp2p_status_handler))
        .route("/api/catp2p/contracts", post(catp2p_contract_handler))
        .route("/api/smp/messages", post(smp_send_handler))
        .route("/api/smp/messages/{queue_id}", get(smp_poll_handler))
        .route("/api/xftp/package", post(xftp_package_handler))
        .with_state(state)
}

async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "vangoghcat-server-rust",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn catp2p_status_handler(State(state): State<AppState>) -> Json<DaemonStatus> {
    let status = state.daemon.get_status().await;
    Json(status)
}

async fn catp2p_contract_handler(
    State(state): State<AppState>,
    Json(req): Json<StorageContractReq>,
) -> impl IntoResponse {
    let mut circle = state.daemon.circle.write().await;
    let ack = handle_contract_request(&req, &state.daemon.identity.node_id, &mut circle);
    (StatusCode::OK, Json(ack))
}

async fn smp_send_handler(
    State(state): State<AppState>,
    Json(payload): Json<SendSmpReq>,
) -> Result<Json<SmpMessage>, StatusCode> {
    let ttl = payload.ttl_seconds.unwrap_or(3600);
    state
        .daemon
        .smp
        .push_message(
            &payload.queue_id,
            &payload.sender_node_id,
            &payload.payload_encrypted_hex,
            ttl,
        )
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn smp_poll_handler(
    State(state): State<AppState>,
    Path(queue_id): Path<String>,
) -> Json<Vec<SmpMessage>> {
    let messages = state.daemon.smp.poll_queue(&queue_id);
    Json(messages)
}

async fn xftp_package_handler(
    Json(req): Json<PackageArtReq>,
) -> Result<Json<PackageArtResp>, StatusCode> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    let data = STANDARD
        .decode(&req.raw_base64)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let (manifest, chunks) = XftpEngine::package(&data, req.chunk_size.unwrap_or(64 * 1024));

    Ok(Json(PackageArtResp {
        chunks_count: chunks.len(),
        manifest,
    }))
}
