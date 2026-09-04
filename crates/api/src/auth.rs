use argon2::password_hash::SaltString;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::rand_core::OsRng,
};
use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::{AdminProfile, AuthResponse};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

/// Session lifetime for admin tokens. Short-lived by design.
pub const SESSION_TTL: Duration = Duration::hours(12);

/// Hash a plaintext password with Argon2id stored in PHC string format.
pub fn hash_password(password: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| {
            tracing::error!("argon2 hashing failed");
            ApiError::internal("hashing password")
        })
}

/// Verify a plaintext password against a stored Argon2id PHC hash.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    iat: i64,
    exp: i64,
}

/// Issue a short-lived signed bearer token for the given admin id.
fn issue_token(jwt_secret: &str, admin_id: Uuid) -> ApiResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: admin_id,
        iat: now.timestamp(),
        exp: (now + SESSION_TTL).timestamp(),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::internal("signing session token"))
}

/// Verify a bearer token and return the admin id it was issued to.
fn verify_token(jwt_secret: &str, token: &str) -> Option<Uuid> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;
    if data.claims.exp < Utc::now().timestamp() {
        return None;
    }
    Some(data.claims.sub)
}

const AUTH_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// Extract the bearer token from a request (no DB access).
pub fn extract_bearer(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(AUTH_HEADER)
        .and_then(|v| v.to_str().ok())
        .and_then(|value| {
            value
                .strip_prefix(BEARER_PREFIX)
                .map(|t| t.trim().to_string())
        })
        .filter(|t| !t.is_empty())
}

/// Identity of an authenticated admin, extracted from a valid bearer token.
/// Presence of this extractor proves the request was authenticated.
pub struct AuthAdmin {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(parts).ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "unauthenticated",
                "missing bearer token",
            )
        })?;

        verify_token(&state.config.jwt_secret, &token)
            .map(|id| AuthAdmin { id })
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "unauthenticated",
                    "invalid or expired token",
                )
            })
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BootstrapRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

struct DbAdmin {
    id: Uuid,
    username: String,
    password_hash: String,
    is_active: bool,
}

/// Create the first administrator. Only succeeds while no admin users exist, so
/// that a fresh deployment can bootstrap credentials without storing them in
/// source. Returns 409 once an admin already exists.
pub async fn bootstrap(
    State(state): State<AppState>,
    Json(req): Json<BootstrapRequest>,
) -> ApiResult<Response> {
    validate_bootstrap_input(&req)?;

    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users")
        .fetch_one(&state.db)
        .await
        .map_err(|_| ApiError::internal("querying admin users"))?;

    if existing > 0 {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "already_bootstrapped",
            "an administrator already exists",
        ));
    }

    let password_hash = hash_password(&req.password)?;
    let admin_id = create_admin(&state.db, &req.username, &password_hash).await?;

    let token = issue_token(&state.config.jwt_secret, admin_id)?;
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            admin: AdminProfile {
                id: admin_id,
                username: req.username,
            },
        }),
    )
        .into_response())
}

fn validate_bootstrap_input(req: &BootstrapRequest) -> ApiResult<()> {
    validate_username(&req.username)?;
    if req.password.len() < 8 {
        return Err(ApiError::validation(
            "password must be at least 8 characters",
            serde_json::json!({"password": "too_short"}),
        ));
    }
    Ok(())
}

fn validate_username(username: &str) -> ApiResult<()> {
    let valid = !username.is_empty()
        && username.chars().count() <= 64
        && username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !valid {
        return Err(ApiError::validation(
            "invalid username",
            serde_json::json!({"username": "must be 1-64 alphanumeric, '_', '-' or '.'"}),
        ));
    }
    Ok(())
}

async fn create_admin(db: &PgPool, username: &str, password_hash: &str) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        "INSERT INTO admin_users (username, password_hash) VALUES ($1, $2) RETURNING id",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(db)
    .await
    .map(|id: Uuid| id)
    .map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::conflict("an administrator with that username already exists")
        } else {
            ApiError::internal("creating admin user")
        }
    })
}

async fn find_admin_by_username(db: &PgPool, username: &str) -> ApiResult<Option<DbAdmin>> {
    let row = sqlx::query_as::<_, (Uuid, String, String, bool)>(
        "SELECT id, username, password_hash, is_active FROM admin_users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(db)
    .await
    .map_err(|_| ApiError::internal("querying admin user"))?;

    Ok(row.map(|(id, username, password_hash, is_active)| DbAdmin {
        id,
        username,
        password_hash,
        is_active,
    }))
}

/// Authenticate an administrator and return a short-lived signed token.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    let Some(admin) = find_admin_by_username(&state.db, &req.username).await? else {
        // Uniform timing/response for unknown user or wrong password.
        return Err(invalid_credentials());
    };

    if !admin.is_active {
        return Err(invalid_credentials());
    }

    if !verify_password(&req.password, &admin.password_hash) {
        return Err(invalid_credentials());
    }

    let token = issue_token(&state.config.jwt_secret, admin.id)?;
    Ok(Json(AuthResponse {
        token,
        admin: AdminProfile {
            id: admin.id,
            username: admin.username,
        },
    }))
}

fn invalid_credentials() -> ApiError {
    // 401, not 404, to avoid leaking whether a username exists.
    ApiError::new(
        StatusCode::UNAUTHORIZED,
        "invalid_credentials",
        "invalid username or password",
    )
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db_err) if db_err.is_unique_violation()
    )
}
