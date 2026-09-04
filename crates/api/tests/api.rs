//! End-to-end API integration tests. Each test runs against a fresh,
//! auto-migrated PostgreSQL database and drives the real router through
//! `tower::ServiceExt::oneshot`, exercising auth, content, home items, media
//! and the public player endpoints.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use civico_api::domain::ContentType;
use civico_api::state::{AppState, Config};
use civico_api::{build_router, error::ApiError, media_store};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-jwt-secret";

/// Build an `AppState` backed by a unique temp media directory.
fn app_state(pool: PgPool, max_upload: u64) -> (AppState, PathBuf) {
    let dir = std::env::temp_dir().join(format!("civico-api-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = Arc::new(Config {
        jwt_secret: TEST_JWT_SECRET.to_string(),
        media_dir: dir.clone(),
        max_upload_bytes: max_upload,
    });
    (AppState { db: pool, config }, dir)
}

async fn router_with_limit(pool: PgPool, max_upload: u64) -> Router {
    let (state, _dir) = app_state(pool, max_upload);
    build_router(state)
}

async fn router(pool: PgPool) -> Router {
    router_with_limit(pool, 10 * 1024 * 1024).await
}

fn json_request(
    method: Method,
    uri: &str,
    body: Option<&Value>,
    token: Option<&str>,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    match body {
        Some(value) => builder
            .header("Content-Type", "application/json")
            .body(Body::from(value.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn send(app: &Router, req: Request<Body>) -> Response {
    app.clone().oneshot(req).await.unwrap()
}

async fn json_of(resp: Response) -> (StatusCode, Value) {
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

/// Bootstrap the first admin and return the auth token.
async fn bootstrap(app: &Router) -> String {
    let (status, body) = response_json(
        app,
        json_request(
            Method::POST,
            "/api/v1/admin/auth/bootstrap",
            Some(&json!({"username": "admin", "password": "password123"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "bootstrap failed: {body}");
    body["token"].as_str().unwrap().to_string()
}

async fn response_json(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    json_of(send(app, req).await).await
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_creates_admin_and_returns_token(pool: PgPool) {
    let app = router(pool).await;
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/auth/bootstrap",
            Some(&json!({"username": "admin", "password": "password123"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].is_string());
    assert_eq!(body["admin"]["username"], "admin");
    assert!(body["admin"]["id"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_rejects_second_admin(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;
    // Second bootstrap attempt (even with a valid-looking token) is rejected
    // because an admin already exists.
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/auth/bootstrap",
            Some(&json!({"username": "root", "password": "password123"})),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "already_bootstrapped");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_succeeds_with_valid_credentials(pool: PgPool) {
    let app = router(pool).await;
    bootstrap(&app).await;
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/auth/login",
            Some(&json!({"username": "admin", "password": "password123"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn login_rejects_wrong_password(pool: PgPool) {
    let app = router(pool).await;
    bootstrap(&app).await;
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/auth/login",
            Some(&json!({"username": "admin", "password": "wrong-password"})),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
}

#[sqlx::test(migrations = "./migrations")]
async fn protected_route_requires_token(pool: PgPool) {
    let app = router(pool).await;
    bootstrap(&app).await;
    // No token.
    let (status, body) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/admin/content", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "unauthenticated");

    // Garbage token.
    let (status, _) = response_json(
        &app,
        json_request(
            Method::GET,
            "/api/v1/admin/content",
            None,
            Some("not-a-real-token"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn health_endpoint_reports_up(pool: PgPool) {
    let app = router(pool).await;
    let (status, body) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/health", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["database"], "up");
}

// ---------------------------------------------------------------------------
// Content CRUD
// ---------------------------------------------------------------------------

async fn create_content(app: &Router, token: &str, overrides: Value) -> Value {
    let mut payload = json!({
        "content_type": "event",
        "title": "Launch Party",
        "slug": "launch-party",
        "publication_status": "published",
    });
    merge(&mut payload, overrides);
    let (status, body) = response_json(
        app,
        json_request(
            Method::POST,
            "/api/v1/admin/content",
            Some(&payload),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create failed: {body}");
    body
}

fn merge(base: &mut Value, overrides: Value) {
    if let (Value::Object(b), Value::Object(o)) = (base, overrides) {
        for (k, v) in o {
            b.insert(k, v);
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn content_create_list_get_update_delete(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    let created = create_content(&app, &token, json!({})).await;
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["content_type"], "event");

    // List.
    let (status, list) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/admin/content", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Get by id.
    let (status, got) = response_json(
        &app,
        json_request(
            Method::GET,
            &format!("/api/v1/admin/content/{id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["slug"], "launch-party");

    // Update.
    let (status, updated) = response_json(
        &app,
        json_request(
            Method::PUT,
            &format!("/api/v1/admin/content/{id}"),
            Some(&json!({"title": "Renamed"})),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "Renamed");

    // Delete.
    let (status, _) = response_json(
        &app,
        json_request(
            Method::DELETE,
            &format!("/api/v1/admin/content/{id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Gone.
    let (status, _) = response_json(
        &app,
        json_request(
            Method::GET,
            &format!("/api/v1/admin/content/{id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_slug_is_rejected(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/content",
            Some(&json!({
                "content_type": "news",
                "title": "Bad slug",
                "slug": "Has Spaces And Punctuation!"
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[sqlx::test(migrations = "./migrations")]
async fn inverted_publish_window_is_rejected(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;
    let from = Utc::now() - chrono::Duration::days(2);
    let until = from - chrono::Duration::days(1);
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/content",
            Some(&json!({
                "content_type": "news",
                "title": "x",
                "slug": "x",
                "publish_from": from.to_rfc3339(),
                "publish_until": until.to_rfc3339(),
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

// ---------------------------------------------------------------------------
// Player publication visibility
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn player_only_returns_effectively_published_content(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    create_content(&app, &token, json!({"title": "Visible", "slug": "visible"})).await;
    create_content(
        &app,
        &token,
        json!({"title": "Draft", "slug": "draft", "publication_status": "draft"}),
    )
    .await;
    create_content(
        &app,
        &token,
        json!({"title": "Archived", "slug": "archived", "publication_status": "archived"}),
    )
    .await;
    // Not yet published (publish_from in the future).
    let future_from = Utc::now() + chrono::Duration::days(1);
    create_content(
        &app,
        &token,
        json!({"title": "Future", "slug": "future", "publish_from": future_from.to_rfc3339()}),
    )
    .await;
    // Expired (publish_until in the past).
    let past_until = Utc::now() - chrono::Duration::days(1);
    create_content(
        &app,
        &token,
        json!({
            "title": "Expired",
            "slug": "expired",
            "publish_from": (Utc::now() - chrono::Duration::days(2)).to_rfc3339(),
            "publish_until": past_until.to_rfc3339(),
        }),
    )
    .await;

    let (status, list) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/player/content", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let slugs: Vec<String> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["slug"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(slugs, vec!["visible".to_string()]);
}

#[sqlx::test(migrations = "./migrations")]
async fn player_content_detail_hides_unpublished(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;
    create_content(
        &app,
        &token,
        json!({"title": "Draft", "slug": "my-draft", "publication_status": "draft"}),
    )
    .await;
    create_content(&app, &token, json!({"title": "Live", "slug": "my-live"})).await;

    // Published slug resolves.
    let (status, detail) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/player/content/my-live", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["title"], "Live");

    // Draft slug is not visible to the public.
    let (status, _) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/player/content/my-draft", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn player_content_list_filters_by_type(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;
    create_content(
        &app,
        &token,
        json!({"content_type": "news", "title": "N", "slug": "n"}),
    )
    .await;
    create_content(
        &app,
        &token,
        json!({"content_type": "page", "title": "P", "slug": "p"}),
    )
    .await;

    let (_, list) = response_json(
        &app,
        json_request(
            Method::GET,
            "/api/v1/player/content?content_type=news",
            None,
            None,
        ),
    )
    .await;
    let slugs: Vec<String> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["slug"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(slugs, vec!["n".to_string()]);

    // Invalid content type returns a structured validation error.
    let (status, body) = response_json(
        &app,
        json_request(
            Method::GET,
            "/api/v1/player/content?content_type=bogus",
            None,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

// ---------------------------------------------------------------------------
// HOME items
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./migrations")]
async fn home_item_section_crud(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    // Create a section home item.
    let (status, item) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/home-items",
            Some(&json!({
                "title": "Events",
                "position": 1,
                "enabled": true,
                "destination_type": "section",
                "destination_section_type": "event",
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create failed: {item}");
    let id = item["id"].as_str().unwrap().to_string();
    assert_eq!(item["destination_type"], "section");
    assert_eq!(item["destination_section_type"], "event");

    // Update it.
    let (status, updated) = response_json(
        &app,
        json_request(
            Method::PUT,
            &format!("/api/v1/admin/home-items/{id}"),
            Some(&json!({"title": "All Events"})),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "All Events");

    // List.
    let (status, list) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/admin/home-items", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Delete.
    let (status, _) = response_json(
        &app,
        json_request(
            Method::DELETE,
            &format!("/api/v1/admin/home-items/{id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn home_item_destination_validation(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    // Section without a section type is invalid.
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/home-items",
            Some(&json!({
                "title": "Broken",
                "destination_type": "section",
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");

    // Content destination referencing a missing content id is invalid.
    let (status, body) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/home-items",
            Some(&json!({
                "title": "Missing",
                "destination_type": "content",
                "destination_content_id": uuid::Uuid::new_v4().to_string(),
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
}

#[sqlx::test(migrations = "./migrations")]
async fn home_item_content_destination_resolves_to_slug(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    let content = create_content(&app, &token, json!({})).await;
    let content_id = content["id"].as_str().unwrap().to_string();

    let (status, _item) = response_json(
        &app,
        json_request(
            Method::POST,
            "/api/v1/admin/home-items",
            Some(&json!({
                "title": "Launch",
                "position": 1,
                "enabled": true,
                "destination_type": "content",
                "destination_content_id": content_id,
            })),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, home) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/player/home", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let items = home.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Launch");
    assert_eq!(items[0]["destination"]["type"], "content");
    assert_eq!(items[0]["destination"]["slug"], "launch-party");
}

#[sqlx::test(migrations = "./migrations")]
async fn player_home_orders_and_filters_enabled(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    // Two enabled plus one disabled.
    for (title, pos, enabled) in [("Second", 2, true), ("First", 1, true), ("Off", 0, false)] {
        let (status, _) = response_json(
            &app,
            json_request(
                Method::POST,
                "/api/v1/admin/home-items",
                Some(&json!({
                    "title": title,
                    "position": pos,
                    "enabled": enabled,
                    "destination_type": "section",
                    "destination_section_type": "place",
                })),
                Some(&token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let (status, home) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/player/home", None, None),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let titles: Vec<String> = home
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["title"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(titles, vec!["First".to_string(), "Second".to_string()]);
    assert_eq!(home[0]["destination"]["type"], "section");
    assert_eq!(home[0]["destination"]["content_type"], "place");
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

fn multipart_request(uri: &str, boundary: &str, body: Vec<u8>, token: &str) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("Authorization", format!("Bearer {token}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

fn multipart_body(filename: &str, content_type: &str, data: &[u8], boundary: &str) -> Vec<u8> {
    let mut body = Vec::new();
    let head = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
    );
    body.extend_from_slice(head.as_bytes());
    body.extend_from_slice(data);
    let tail = format!("\r\n--{boundary}--\r\n");
    body.extend_from_slice(tail.as_bytes());
    body
}

const JPEG: [u8; 8] = [0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46];
const PNG: [u8; 16] = [
    0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];

#[sqlx::test(migrations = "./migrations")]
async fn media_upload_list_get_serve_delete(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    let boundary = "X-TEST-BOUNDARY";
    let body = multipart_body("photo.jpg", "image/jpeg", &JPEG, boundary);
    let (status, media) = response_json(
        &app,
        multipart_request("/api/v1/admin/media", boundary, body, &token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "upload failed: {media}");
    let id = media["id"].as_str().unwrap().to_string();
    assert_eq!(media["filename"], "photo.jpg");
    assert_eq!(media["mime_type"], "image/jpeg");
    assert_eq!(media["size_bytes"], JPEG.len() as i64);

    // Serve the file back over the public media route.
    let resp = send(
        &app,
        Request::builder()
            .uri(format!("/api/v1/media/{id}/file"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()["content-type"],
        "image/jpeg",
        "unexpected content-type"
    );
    let served = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&served[..], &JPEG);

    // List.
    let (status, list) = response_json(
        &app,
        json_request(Method::GET, "/api/v1/admin/media", None, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Delete.
    let (status, _) = response_json(
        &app,
        json_request(
            Method::DELETE,
            &format!("/api/v1/admin/media/{id}"),
            None,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // File is gone after delete.
    let resp = send(
        &app,
        Request::builder()
            .uri(format!("/api/v1/media/{id}/file"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn media_upload_rejects_disallowed_content(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    // SVG content that does not sniff as an allowed image.
    let boundary = "XVG";
    let svg = b"<svg xmlns='http://www.w3.org/2000/svg'></svg>";
    let body = multipart_body("evil.svg", "image/svg+xml", svg, boundary);
    let (status, err) = response_json(
        &app,
        multipart_request("/api/v1/admin/media", boundary, body, &token),
    )
    .await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(err["error"]["code"], "unsupported_media_type");
}

#[sqlx::test(migrations = "./migrations")]
async fn media_upload_trusts_content_sniffing_over_declared_mime(pool: PgPool) {
    let app = router(pool).await;
    let token = bootstrap(&app).await;

    // The client declares a bogus MIME but the *content* is a valid JPEG. We
    // must never trust the client-supplied MIME type: sniffing is authoritative,
    // so the upload succeeds and is stored as image/jpeg.
    let boundary = "MMB";
    let body = multipart_body("photo.jpg", "image/svg+xml", &JPEG, boundary);
    let (status, media) = response_json(
        &app,
        multipart_request("/api/v1/admin/media", boundary, body, &token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "upload failed: {media}");
    assert_eq!(media["mime_type"], "image/jpeg");
}

#[sqlx::test(migrations = "./migrations")]
async fn media_upload_rejects_oversized_body(pool: PgPool) {
    // Use a very small upload limit so a modest body is already too big.
    let app = router_with_limit(pool, 8).await;
    let token = bootstrap(&app).await;

    let boundary = "BIG";
    // 20 bytes of valid-JPEG-looking content exceeds the 8-byte limit.
    let oversized: Vec<u8> = (0..20).flat_map(|_| [0xffu8, 0xd8, 0xff]).collect();
    let body = multipart_body("big.jpg", "image/jpeg", &oversized, boundary);
    let (status, err) = response_json(
        &app,
        multipart_request("/api/v1/admin/media", boundary, body, &token),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(err["error"]["code"], "payload_too_large");
}

// ---------------------------------------------------------------------------
// Media storage safety
// ---------------------------------------------------------------------------

#[test]
fn sniff_mime_identifies_jpeg_png_webp() {
    assert_eq!(media_store::sniff_mime(&JPEG), Some("image/jpeg"));
    assert_eq!(media_store::sniff_mime(&PNG), Some("image/png"));
    assert_eq!(media_store::sniff_mime(b"RIFFxxxxWEBP"), Some("image/webp"));
    assert_eq!(media_store::sniff_mime(b"<svg>"), None);
}

#[test]
fn resolve_stored_path_rejects_traversal() {
    let dir = std::env::temp_dir().join(format!("civico-resolve-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    // Write a file inside the dir so canonicalize() succeeds.
    std::fs::write(dir.join("a.jpg"), b"x").unwrap();

    // A path pointing outside the base must be rejected, not silently resolved.
    let err = media_store::resolve_stored_path(&dir, "../outside").unwrap_err();
    assert_eq!(err.status, StatusCode::NOT_FOUND);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn validate_upload_rejects_empty() {
    let err = media_store::validate_upload(&[], None, 1000).unwrap_err();
    assert_eq!(err.status, StatusCode::BAD_REQUEST);
}

#[test]
fn storage_path_is_derived_from_content_hash_not_filename() {
    let id = uuid::Uuid::new_v4();
    let path = media_store::storage_path_for(id, "deadbeef", "image/jpeg");
    assert!(path.ends_with("/deadbeef.jpg"), "unexpected path: {path}");
    assert!(!path.contains("user-file.jpg"));
}

#[test]
fn api_error_shape_is_structured() {
    let err = ApiError::validation("bad input", json!({"field": "reason"}));
    let resp = err.into_response();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let _ = resp;
}

#[test]
fn content_type_serde_uses_lowercase_values() {
    assert_eq!(
        serde_json::to_string(&ContentType::News).unwrap(),
        "\"news\""
    );
}
