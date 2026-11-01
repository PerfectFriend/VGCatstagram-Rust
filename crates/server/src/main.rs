//! Binary entry point for the sovereign VanGoghCat server daemon.

use std::sync::Arc;
use tracing::info;
use vangoghcat_crypto::CatNodeIdentity;
use vangoghcat_p2p::daemon::CatP2PDaemon;
use vangoghcat_server::build_app;
use vangoghcat_storage::StorageDb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8765".into());
    let addr = format!("0.0.0.0:{}", port);

    let identity = CatNodeIdentity::generate();
    info!("Initializing Van Gogh CryptoCats Daemon v2.0 (Rust Core)");
    info!("Assigned Sovereign CatNodeID: {}", identity.node_id);

    let storage_dir = dirs_or_fallback();
    let db_path = storage_dir.join("vgc_sled_db");
    let db = Arc::new(StorageDb::open(db_path)?);

    let daemon = Arc::new(CatP2PDaemon::new(identity, db));
    let app = build_app(daemon);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("VanGoghCat Axum Core Server running at http://{}", addr);
    info!("WebSocket Signaling Relay available at ws://{}/ws/signaling", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

fn dirs_or_fallback() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let p = std::path::PathBuf::from(home).join(".feline-p2p");
        let _ = std::fs::create_dir_all(&p);
        p
    } else {
        std::env::temp_dir().join("feline-p2p")
    }
}
