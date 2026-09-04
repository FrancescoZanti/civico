//! Concrete persistence functions. No trait abstractions: these are plain
//! `async fn`s operating on the pool, chosen for clarity over indirection.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ContentType, HomeDestinationType, PublicationStatus};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ContentRow {
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HomeItemRow {
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

const HOME_ITEM_COLUMNS: &str = "id, title, subtitle, icon, image_id, position, enabled, \
     destination_type, destination_section_type, destination_content_id";

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MediaRow {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

// ---------------------------------------------------------------- content

const CONTENT_COLUMNS: &str = "id, content_type, title, slug, excerpt, body, \
     publication_status, publish_from, publish_until, published_at, image_id";

pub async fn content_get(db: &PgPool, id: Uuid) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "SELECT {CONTENT_COLUMNS} FROM content WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn content_get_by_slug(
    db: &PgPool,
    slug: &str,
) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "SELECT {CONTENT_COLUMNS} FROM content WHERE slug = $1"
    ))
    .bind(slug)
    .fetch_optional(db)
    .await
}

pub async fn content_list(db: &PgPool) -> Result<Vec<ContentRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "SELECT {CONTENT_COLUMNS} FROM content ORDER BY updated_at DESC"
    ))
    .fetch_all(db)
    .await
}

pub async fn content_list_by_type(
    db: &PgPool,
    content_type: ContentType,
) -> Result<Vec<ContentRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "SELECT {CONTENT_COLUMNS} FROM content WHERE content_type = $1 ORDER BY updated_at DESC"
    ))
    .bind(content_type)
    .fetch_all(db)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn content_create(
    db: &PgPool,
    content_type: ContentType,
    title: &str,
    slug: &str,
    excerpt: Option<&str>,
    body: Option<&str>,
    data: serde_json::Value,
    publication_status: PublicationStatus,
    publish_from: Option<DateTime<Utc>>,
    publish_until: Option<DateTime<Utc>>,
    image_id: Option<Uuid>,
    created_by: Uuid,
) -> Result<ContentRow, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "INSERT INTO content \
            (content_type, title, slug, excerpt, body, data, publication_status, \
             publish_from, publish_until, image_id, created_by, published_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, \
                 CASE WHEN $7 = 'published'::publication_status THEN now() ELSE NULL END) \
         RETURNING {CONTENT_COLUMNS}"
    ))
    .bind(content_type)
    .bind(title)
    .bind(slug)
    .bind(excerpt)
    .bind(body)
    .bind(data)
    .bind(publication_status)
    .bind(publish_from)
    .bind(publish_until)
    .bind(image_id)
    .bind(created_by)
    .fetch_one(db)
    .await
}

/// Set `published_at` when moving to published, clear otherwise.
async fn content_set_published_at(
    db: &PgPool,
    id: Uuid,
    status: PublicationStatus,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE content SET published_at = CASE \
            WHEN $2 = 'published'::publication_status THEN COALESCE(published_at, now()) \
            ELSE NULL END \
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .execute(db)
    .await
    .map(|_| ())
}

#[allow(clippy::too_many_arguments)]
pub async fn content_update(
    db: &PgPool,
    id: Uuid,
    title: Option<&str>,
    slug: Option<&str>,
    excerpt: Option<&str>,
    body: Option<&str>,
    data: Option<serde_json::Value>,
    publication_status: Option<PublicationStatus>,
    publish_from: Option<DateTime<Utc>>,
    publish_until: Option<DateTime<Utc>>,
    image_id: Option<Uuid>,
) -> Result<Option<ContentRow>, sqlx::Error> {
    let existing = content_get(db, id).await?;
    let Some(current) = existing else {
        return Ok(None);
    };

    let new_status = publication_status.unwrap_or(current.publication_status);
    let new_title = title.unwrap_or(&current.title);
    let new_slug = slug.unwrap_or(&current.slug);
    let new_excerpt = excerpt.or(current.excerpt.as_deref());
    let new_body = body.or(current.body.as_deref());
    let new_data = data.unwrap_or(serde_json::Value::Null);

    let row = sqlx::query_as::<_, ContentRow>(&format!(
        "UPDATE content SET \
            title = $2, slug = $3, \
            excerpt = COALESCE($4, excerpt), \
            body = COALESCE($5, body), \
            data = CASE WHEN $6 = 'null'::jsonb THEN data ELSE $6 END, \
            publication_status = $7, \
            publish_from = COALESCE($8, publish_from), \
            publish_until = COALESCE($9, publish_until), \
            image_id = COALESCE($10, image_id), \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {CONTENT_COLUMNS}"
    ))
    .bind(id)
    .bind(new_title)
    .bind(new_slug)
    .bind(new_excerpt)
    .bind(new_body)
    .bind(new_data)
    .bind(new_status)
    .bind(publish_from)
    .bind(publish_until)
    .bind(image_id)
    .fetch_optional(db)
    .await?;

    if let Some(updated) = &row
        && updated.publication_status != current.publication_status
    {
        content_set_published_at(db, id, updated.publication_status).await?;
    }

    // Re-read to fetch the freshest published_at.
    content_get(db, id).await
}

pub async fn content_delete(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ------------------------------------------------- public / player queries

const PUBLISHED_FILTER: &str = "publication_status = 'published'::publication_status \
     AND (publish_from IS NULL OR publish_from <= now()) \
     AND (publish_until IS NULL OR publish_until > now())";

/// Fetch the slug of a content row (used to resolve HOME content destinations).
pub async fn content_slug(db: &PgPool, id: Uuid) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT slug FROM content WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
}

/// List effectively-published content, optionally filtered by content type.
pub async fn content_list_published(
    db: &PgPool,
    content_type: Option<ContentType>,
) -> Result<Vec<ContentRow>, sqlx::Error> {
    match content_type {
        Some(ct) => {
            sqlx::query_as::<_, ContentRow>(&format!(
                "SELECT {CONTENT_COLUMNS} FROM content \
                 WHERE {PUBLISHED_FILTER} AND content_type = $1 \
                 ORDER BY published_at DESC NULLS LAST, updated_at DESC"
            ))
            .bind(ct)
            .fetch_all(db)
            .await
        }
        None => {
            sqlx::query_as::<_, ContentRow>(&format!(
                "SELECT {CONTENT_COLUMNS} FROM content \
                 WHERE {PUBLISHED_FILTER} \
                 ORDER BY published_at DESC NULLS LAST, updated_at DESC"
            ))
            .fetch_all(db)
            .await
        }
    }
}

/// Fetch effectively-published content by slug for the public player.
pub async fn content_get_public_by_slug(
    db: &PgPool,
    slug: &str,
) -> Result<Option<ContentRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentRow>(&format!(
        "SELECT {CONTENT_COLUMNS} FROM content WHERE {PUBLISHED_FILTER} AND slug = $1"
    ))
    .bind(slug)
    .fetch_optional(db)
    .await
}

// ---------------------------------------------------------------- home items

pub async fn home_item_get(db: &PgPool, id: Uuid) -> Result<Option<HomeItemRow>, sqlx::Error> {
    sqlx::query_as::<_, HomeItemRow>(&format!(
        "SELECT {HOME_ITEM_COLUMNS} FROM home_items WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn home_item_list(db: &PgPool) -> Result<Vec<HomeItemRow>, sqlx::Error> {
    sqlx::query_as::<_, HomeItemRow>(&format!(
        "SELECT {HOME_ITEM_COLUMNS} FROM home_items ORDER BY position ASC, created_at ASC"
    ))
    .fetch_all(db)
    .await
}

pub async fn home_item_list_enabled(db: &PgPool) -> Result<Vec<HomeItemRow>, sqlx::Error> {
    sqlx::query_as::<_, HomeItemRow>(&format!(
        "SELECT {HOME_ITEM_COLUMNS} FROM home_items WHERE enabled = true \
         ORDER BY position ASC, created_at ASC"
    ))
    .fetch_all(db)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn home_item_create(
    db: &PgPool,
    title: &str,
    subtitle: Option<&str>,
    icon: Option<&str>,
    image_id: Option<Uuid>,
    position: i32,
    enabled: bool,
    destination_type: HomeDestinationType,
    destination_section_type: Option<ContentType>,
    destination_content_id: Option<Uuid>,
) -> Result<HomeItemRow, sqlx::Error> {
    sqlx::query_as::<_, HomeItemRow>(&format!(
        "INSERT INTO home_items \
            (title, subtitle, icon, image_id, position, enabled, destination_type, \
             destination_section_type, destination_content_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         RETURNING {HOME_ITEM_COLUMNS}"
    ))
    .bind(title)
    .bind(subtitle)
    .bind(icon)
    .bind(image_id)
    .bind(position)
    .bind(enabled)
    .bind(destination_type)
    .bind(destination_section_type)
    .bind(destination_content_id)
    .fetch_one(db)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn home_item_update(
    db: &PgPool,
    id: Uuid,
    title: Option<&str>,
    subtitle: Option<&str>,
    icon: Option<&str>,
    image_id: Option<Uuid>,
    position: Option<i32>,
    enabled: Option<bool>,
    destination_type: HomeDestinationType,
    destination_section_type: Option<ContentType>,
    destination_content_id: Option<Uuid>,
) -> Result<Option<HomeItemRow>, sqlx::Error> {
    sqlx::query_as::<_, HomeItemRow>(&format!(
        "UPDATE home_items SET \
            title = COALESCE($2, title), \
            subtitle = COALESCE($3, subtitle), \
            icon = COALESCE($4, icon), \
            image_id = COALESCE($5, image_id), \
            position = COALESCE($6, position), \
            enabled = COALESCE($7, enabled), \
            destination_type = $8, \
            destination_section_type = $9, \
            destination_content_id = $10, \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING {HOME_ITEM_COLUMNS}"
    ))
    .bind(id)
    .bind(title)
    .bind(subtitle)
    .bind(icon)
    .bind(image_id)
    .bind(position)
    .bind(enabled)
    .bind(destination_type)
    .bind(destination_section_type)
    .bind(destination_content_id)
    .fetch_optional(db)
    .await
}

pub async fn home_item_delete(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM home_items WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------- media

pub async fn media_get(db: &PgPool, id: Uuid) -> Result<Option<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        "SELECT id, filename, mime_type, size_bytes, storage_path, width, height \
         FROM media WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn media_list(db: &PgPool) -> Result<Vec<MediaRow>, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        "SELECT id, filename, mime_type, size_bytes, storage_path, width, height \
         FROM media ORDER BY created_at DESC",
    )
    .fetch_all(db)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn media_create(
    db: &PgPool,
    filename: &str,
    mime_type: &str,
    size_bytes: i64,
    storage_path: &str,
    width: Option<i32>,
    height: Option<i32>,
    created_by: Uuid,
) -> Result<MediaRow, sqlx::Error> {
    sqlx::query_as::<_, MediaRow>(
        "INSERT INTO media (filename, mime_type, size_bytes, storage_path, width, height, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, filename, mime_type, size_bytes, storage_path, width, height",
    )
    .bind(filename)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(storage_path)
    .bind(width)
    .bind(height)
    .bind(created_by)
    .fetch_one(db)
    .await
}

pub async fn media_delete(db: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM media WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected() > 0)
}
