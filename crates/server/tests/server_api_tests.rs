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
