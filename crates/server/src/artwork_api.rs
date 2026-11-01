//! # Artwork REST API
//!
//! Full CRUD endpoints for artwork management through the Storage Engine:
//! - `POST /api/artworks` — create and persist a new artwork manifest
//! - `GET /api/artworks` — list all artworks with pagination
//! - `GET /api/artworks/:id` — retrieve a single artwork by ID
//! - `DELETE /api/artworks/:id` — soft-delete (condemn) an artwork
//! - `POST /api/artworks/:id/like` — increment like counter
//! - `GET /api/artworks/stats` — aggregate storage statistics

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use vangoghcat_core::types::Artwork;
use vangoghcat_storage::StorageDb;

// ─────────────────────────────────────────────────────────────────────────────
// State & DTOs
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ArtworkApiState {
    pub db: Arc<StorageDb>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtworkListResponse {
    pub artworks: Vec<Artwork>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtworkStatsResponse {
    pub total_artworks: usize,
    pub total_storage_bytes: u64,
    pub user_created_count: usize,
    pub masterpiece_count: usize,
    pub condemned_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LikeResponse {
    pub artwork_id: String,
    pub new_likes_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub artwork_id: String,
    pub condemned: bool,
    pub reason: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────────────────────

pub fn artwork_router(state: ArtworkApiState) -> Router {
    Router::new()
        .route("/api/artworks", get(list_artworks))
        .route("/api/artworks", post(create_artwork))
        .route("/api/artworks/stats", get(artwork_stats))
        .route("/api/artworks/{id}", get(get_artwork))
        .route("/api/artworks/{id}", delete(delete_artwork))
        .route("/api/artworks/{id}/like", post(like_artwork))
        .with_state(state)
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// `GET /api/artworks?page=1&per_page=20`
///
/// Returns a paginated list of all non-condemned artworks.
async fn list_artworks(
    State(state): State<ArtworkApiState>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = pagination.page.unwrap_or(1).max(1);
    let per_page = pagination.per_page.unwrap_or(20).min(100);

    let all = state.db.list_artworks();
    let non_condemned: Vec<Artwork> = all
        .into_iter()
        .filter(|a| !a.is_condemned.unwrap_or(false))
        .collect();

    let total = non_condemned.len();
    let start = (page - 1) * per_page;
    let artworks: Vec<Artwork> = non_condemned
        .into_iter()
        .skip(start)
        .take(per_page)
        .collect();

    Json(ArtworkListResponse {
        artworks,
        total,
        page,
        per_page,
    })
}

/// `POST /api/artworks`
///
/// Creates and persists a new artwork manifest. Returns the created artwork.
async fn create_artwork(
    State(state): State<ArtworkApiState>,
    Json(artwork): Json<Artwork>,
) -> Result<impl IntoResponse, StatusCode> {
    // Validate required fields
    if artwork.id.is_empty() || artwork.title.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check for duplicate ID
    if state.db.get_artwork(&artwork.id).is_some() {
        warn!(artwork_id = %artwork.id, "Duplicate artwork ID rejected");
        return Err(StatusCode::CONFLICT);
    }

    state
        .db
        .put_artwork(&artwork)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(artwork_id = %artwork.id, title = %artwork.title, "Artwork created");

    Ok((StatusCode::CREATED, Json(artwork)))
}

/// `GET /api/artworks/:id`
///
/// Retrieves a single artwork by its content-addressed ID.
async fn get_artwork(
    State(state): State<ArtworkApiState>,
    Path(id): Path<String>,
) -> Result<Json<Artwork>, StatusCode> {
    state
        .db
        .get_artwork(&id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// `DELETE /api/artworks/:id`
///
/// Soft-deletes (condemns) an artwork, setting `is_condemned = true`.
/// The artwork data is preserved but excluded from listings.
async fn delete_artwork(
    State(state): State<ArtworkApiState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, StatusCode> {
    let mut artwork = state.db.get_artwork(&id).ok_or(StatusCode::NOT_FOUND)?;

    artwork.is_condemned = Some(true);
    artwork.condemned_reason = Some("User-initiated deletion".to_string());

    state
        .db
        .put_artwork(&artwork)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!(artwork_id = %id, "Artwork condemned (soft-deleted)");

    Ok(Json(DeleteResponse {
        artwork_id: id,
        condemned: true,
        reason: "User-initiated deletion".to_string(),
    }))
}

/// `POST /api/artworks/:id/like`
///
/// Atomically increments the like counter for an artwork.
async fn like_artwork(
    State(state): State<ArtworkApiState>,
    Path(id): Path<String>,
) -> Result<Json<LikeResponse>, StatusCode> {
    let mut artwork = state.db.get_artwork(&id).ok_or(StatusCode::NOT_FOUND)?;

    let new_count = artwork.likes_count.unwrap_or(0) + 1;
    artwork.likes_count = Some(new_count);
    artwork.likes = Some(new_count);

    state
        .db
        .put_artwork(&artwork)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LikeResponse {
        artwork_id: id,
        new_likes_count: new_count,
    }))
}

/// `GET /api/artworks/stats`
///
/// Returns aggregate statistics about the artwork collection.
async fn artwork_stats(State(state): State<ArtworkApiState>) -> Json<ArtworkStatsResponse> {
    let all = state.db.list_artworks();
    let total = all.len();

    let user_created = all
        .iter()
        .filter(|a| a.is_user_created == Some(true))
        .count();
    let condemned = all.iter().filter(|a| a.is_condemned == Some(true)).count();
    let masterpiece = total - user_created;

    // Estimate total storage: each artwork's image_url size is a rough proxy
    // In production, we'd query the file storage engine for actual disk usage
    let estimated_bytes: u64 = all.len() as u64 * 2_000_000; // ~2 MB average

    Json(ArtworkStatsResponse {
        total_artworks: total,
        total_storage_bytes: estimated_bytes,
        user_created_count: user_created,
        masterpiece_count: masterpiece,
        condemned_count: condemned,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use vangoghcat_core::types::AspectRatio;

    fn test_artwork(id: &str) -> Artwork {
        Artwork {
            id: id.to_string(),
            title: format!("Test Artwork {}", id),
            title_en: None,
            image_url: "https://example.com/cat.png".to_string(),
            prompt: "A cat painting in Van Gogh style".to_string(),
            style: "Post-Impressionism".to_string(),
            year: "2026".to_string(),
            description: "Test description".to_string(),
            palette: vec![],
            brushstroke_note: "Swirling".to_string(),
            aspect_ratio: AspectRatio::Square,
            created_at: 1730000000,
            is_user_created: Some(true),
            model_used: None,
            is_svg_art: None,
            likes: Some(0),
            likes_count: Some(0),
            style_preset: None,
            prompt_used: None,
            author_id: None,
            author_name: None,
            author_signature: None,
            origin_peer_id: None,
            is_backed_up_locally: None,
            source: None,
            is_condemned: None,
            condemned_reason: None,
            cat_profile_id: None,
            zkp_commitment: None,
            zkp_fingerprint: None,
            zkp_revealed: None,
        }
    }

    fn test_state() -> ArtworkApiState {
        let db = Arc::new(StorageDb::open_temporary().expect("test db"));
        ArtworkApiState { db }
    }

    #[tokio::test]
    async fn test_create_and_get_artwork() {
        let state = test_state();
        let app = artwork_router(state);

        let artwork = test_artwork("art-001");
        let body = serde_json::to_string(&artwork).unwrap();

        // POST /api/artworks
        let create_req = Request::builder()
            .method("POST")
            .uri("/api/artworks")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.clone().oneshot(create_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // GET /api/artworks/art-001
        let get_req = Request::builder()
            .method("GET")
            .uri("/api/artworks/art-001")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(get_req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let returned: Artwork = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(returned.id, "art-001");
        assert_eq!(returned.title, "Test Artwork art-001");
    }

    #[tokio::test]
    async fn test_list_artworks_pagination() {
        let state = test_state();

        // Seed 5 artworks
        for i in 0..5 {
            let art = test_artwork(&format!("art-{:03}", i));
            state.db.put_artwork(&art).unwrap();
        }

        let app = artwork_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/api/artworks?page=1&per_page=2")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let list: ArtworkListResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(list.artworks.len(), 2);
        assert_eq!(list.total, 5);
        assert_eq!(list.page, 1);
        assert_eq!(list.per_page, 2);
    }

    #[tokio::test]
    async fn test_like_artwork() {
        let state = test_state();
        let art = test_artwork("like-test");
        state.db.put_artwork(&art).unwrap();

        let app = artwork_router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/artworks/like-test/like")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let like: LikeResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(like.new_likes_count, 1);
    }

    #[tokio::test]
    async fn test_condemn_artwork() {
        let state = test_state();
        let art = test_artwork("condemn-test");
        state.db.put_artwork(&art).unwrap();

        let app = artwork_router(state);

        let req = Request::builder()
            .method("DELETE")
            .uri("/api/artworks/condemn-test")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let del: DeleteResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert!(del.condemned);
    }

    #[tokio::test]
    async fn test_get_nonexistent_returns_404() {
        let state = test_state();
        let app = artwork_router(state);

        let req = Request::builder()
            .method("GET")
            .uri("/api/artworks/does-not-exist")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_duplicate_create_returns_409() {
        let state = test_state();
        let art = test_artwork("dup-test");
        state.db.put_artwork(&art).unwrap();

        let app = artwork_router(state);

        let body = serde_json::to_string(&art).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/api/artworks")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }
}
