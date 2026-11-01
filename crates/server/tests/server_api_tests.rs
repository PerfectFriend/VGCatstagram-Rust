use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;
use vangoghcat_crypto::CatNodeIdentity;
use vangoghcat_p2p::daemon::CatP2PDaemon;
use vangoghcat_server::build_app;
use vangoghcat_storage::StorageDb;

fn setup_test_app() -> axum::Router {
    let identity = CatNodeIdentity::generate();
    let db = Arc::new(StorageDb::open_temporary().expect("test sled"));
    let daemon = Arc::new(CatP2PDaemon::new(identity, db));
    build_app(daemon)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "vangoghcat-server-rust");
}

#[tokio::test]
async fn test_catp2p_status_endpoint() {
    let app = setup_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/catp2p/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["nodeId"].as_str().unwrap().starts_with("catnode1"));
    assert_eq!(json["circlePeersCount"], 0);
    assert_eq!(json["isOnline"], true);
}

#[tokio::test]
async fn test_smp_send_and_poll() {
    let app = setup_test_app();

    // 1. Send SMP Message
    let send_payload = serde_json::json!({
        "queue_id": "smpq_integration_test",
        "sender_node_id": "catnode1alice",
        "payload_encrypted_hex": "cafebabe123456",
        "ttl_seconds": 1800
    });

    let send_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/smp/messages")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&send_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(send_resp.status(), StatusCode::OK);

    // 2. Poll SMP Messages
    let poll_resp = app
        .oneshot(
            Request::builder()
                .uri("/api/smp/messages/smpq_integration_test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(poll_resp.status(), StatusCode::OK);
    let body = poll_resp.into_body().collect().await.unwrap().to_bytes();
    let messages: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["payload_encrypted_hex"], "cafebabe123456");
}

#[tokio::test]
async fn test_spa_static_serving_and_fallback() {
    let identity = CatNodeIdentity::generate();
    let db = Arc::new(StorageDb::open_temporary().expect("test sled"));
    let daemon = Arc::new(CatP2PDaemon::new(identity, db));

    // Create a temporary directory simulating the dist/ folder
    let temp_dir = std::env::temp_dir().join(format!("vgc_spa_test_{}", uuid::Uuid::new_v4()));
    let assets_dir = temp_dir.join("assets");
    std::fs::create_dir_all(&assets_dir).unwrap();

    let index_content =
        "<!DOCTYPE html><html><body><div id=\"root\">Van Gogh Cats SPA</div></body></html>";
    std::fs::write(temp_dir.join("index.html"), index_content).unwrap();

    let js_content = "console.log('VanGoghCat Sovereign SPA loaded');";
    std::fs::write(assets_dir.join("bundle.js"), js_content).unwrap();

    let app = vangoghcat_server::build_app_with_web(daemon, Some(temp_dir.clone()));

    // 1. Root route / -> returns index.html
    let root_resp = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(root_resp.status(), StatusCode::OK);
    let root_body = root_resp.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&root_body).contains("Van Gogh Cats SPA"));

    // 2. Static asset /assets/bundle.js -> returns JS file
    let asset_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/assets/bundle.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(asset_resp.status(), StatusCode::OK);
    let asset_body = asset_resp.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&asset_body).contains("VanGoghCat Sovereign SPA loaded"));

    // 3. Client-side SPA navigation route (e.g. /gallery/view/42) -> fallbacks to index.html
    let spa_nav_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/gallery/view/42")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spa_nav_resp.status(), StatusCode::OK);
    let spa_nav_body = spa_nav_resp.into_body().collect().await.unwrap().to_bytes();
    assert!(String::from_utf8_lossy(&spa_nav_body).contains("Van Gogh Cats SPA"));

    // 4. API routes still take precedence over SPA fallback
    let health_resp = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(health_resp.status(), StatusCode::OK);

    // Clean up temporary test files
    let _ = std::fs::remove_dir_all(temp_dir);
}
