use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{ContentType, HomeDestinationType, PublicationStatus};

/// A field-level validation failure, mapped to `details` in an error response.
pub fn field_error(field: &str, reason: &str) -> serde_json::Value {
    serde_json::json!({ field: reason })
}

// ---------------------------------------------------------------- auth DTOs

#[derive(Debug, Serialize)]
pub struct AdminProfile {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub admin: AdminProfile,
}

// ---------------------------------------------------------------- content

#[derive(Debug, Serialize)]
pub struct ContentDto {
    pub id: Uuid,
    pub content_type: ContentType,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub body: Option<String>,
    pub publication_status: PublicationStatus,
    pub publish_from: Option<DateTime<Utc>>,
    pub publish_until: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ContentCreate {
    pub content_type: ContentType,
    pub title: String,
    pub slug: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub publication_status: Option<PublicationStatus>,
    #[serde(default)]
    pub publish_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub publish_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ContentUpdate {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub publication_status: Option<PublicationStatus>,
    #[serde(default)]
    pub publish_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub publish_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub image_id: Option<Uuid>,
}

// ---------------------------------------------------------------- home items

#[derive(Debug, Serialize)]
pub struct HomeItemDto {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub image_id: Option<Uuid>,
    pub position: i32,
    pub enabled: bool,
    pub destination_type: HomeDestinationType,
    pub destination_section_type: Option<ContentType>,
    pub destination_content_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct HomeItemCreate {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub image_id: Option<Uuid>,
    #[serde(default = "default_position")]
    pub position: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub destination_type: HomeDestinationType,
    #[serde(default)]
    pub destination_section_type: Option<ContentType>,
    #[serde(default)]
    pub destination_content_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct HomeItemUpdate {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub image_id: Option<Uuid>,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub destination_type: Option<HomeDestinationType>,
    #[serde(default)]
    pub destination_section_type: Option<ContentType>,
    #[serde(default)]
    pub destination_content_id: Option<Uuid>,
}

fn default_position() -> i32 {
    0
}

fn default_true() -> bool {
    true
}

// ---------------------------------------------------------------- media

#[derive(Debug, Serialize)]
pub struct MediaDto {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    /// Public URL to fetch the stored file content.
    pub url: String,
}

impl MediaDto {
    /// Build a public-facing media record from a repo row, deriving the fetch URL.
    pub fn from_row(row: crate::repo::MediaRow, url_for: impl FnOnce(Uuid) -> String) -> Self {
        let url = url_for(row.id);
        MediaDto {
            id: row.id,
            filename: row.filename,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            width: row.width,
            height: row.height,
            url,
        }
    }
}

// ---------------------------------------------------------------- player

/// A HOME button as returned to the public player: only the fields the player
/// needs, no admin-only metadata.
#[derive(Debug, Serialize)]
pub struct PlayerHomeItem {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub image_url: Option<String>,
    pub destination: PlayerDestination,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PlayerDestination {
    /// Navigate to a whole section listing of a content type.
    Section { content_type: ContentType },
    /// Navigate to a specific content item by its slug.
    Content { slug: String },
}

/// Minimal content presented to the public player.
#[derive(Debug, Serialize)]
pub struct PlayerContent {
    pub id: Uuid,
    pub content_type: ContentType,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub body: Option<String>,
    pub image_url: Option<String>,
}
