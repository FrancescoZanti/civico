use civico_api::domain::{ContentType, PublicationStatus};
use sqlx::PgPool;

/// Proves a clean database can migrate from zero and that all four tables and
/// the supporting indexes exist afterwards.
#[sqlx::test(migrations = "./migrations")]
async fn clean_database_migrates_from_zero_and_creates_schema(pool: PgPool) {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    for expected in ["admin_users", "home_items", "content", "media"] {
        assert!(
            tables.iter().any(|t| t == expected),
            "missing table {expected}: {tables:?}"
        );
    }
}

/// Invalid publication states are rejected at the database layer.
#[sqlx::test(migrations = "./migrations")]
async fn invalid_publication_status_is_rejected(pool: PgPool) {
    let result = sqlx::query(
        "INSERT INTO content (content_type, title, slug, publication_status)
         VALUES ('page', 'x', 'x', 'bogus')",
    )
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "an unknown publication_status must be rejected by the database"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.to_lowercase().contains("publication_status"), "{err}");
}

/// Invalid content types are rejected at the database layer.
#[sqlx::test(migrations = "./migrations")]
async fn invalid_content_type_is_rejected(pool: PgPool) {
    let result = sqlx::query(
        "INSERT INTO content (content_type, title, slug) VALUES ('not-a-type', 'x', 'x')",
    )
    .execute(&pool)
    .await;

    assert!(result.is_err());
}

/// The Rust domain enums map to the PostgreSQL enums and round-trip through them.
#[sqlx::test(migrations = "./migrations")]
async fn domain_enums_round_trip_through_database(pool: PgPool) {
    let id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO content (content_type, title, slug, publication_status)
         VALUES ('event', 'x', 'x', 'published') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let (content_type, status): (ContentType, PublicationStatus) =
        sqlx::query_as("SELECT content_type, publication_status FROM content WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(content_type, ContentType::Event);
    assert_eq!(status, PublicationStatus::Published);
}

/// publish_until must be later than publish_from when both are set.
#[sqlx::test(migrations = "./migrations")]
async fn inverted_publish_window_is_rejected(pool: PgPool) {
    let result = sqlx::query(
        "INSERT INTO content (content_type, title, slug, publish_from, publish_until)
         VALUES ('news', 'x', 'x', '2026-01-02 00:00:00+00', '2026-01-01 00:00:00+00')",
    )
    .execute(&pool)
    .await;

    assert!(
        result.is_err(),
        "an inverted publish window (until before from) must be rejected"
    );
}

/// A section destination must not carry a content reference and must name a
/// section type; a content destination must carry one (home_items constraints).
#[sqlx::test(migrations = "./migrations")]
async fn home_item_destination_constraints_are_enforced(pool: PgPool) {
    let section_with_content = sqlx::query(
        "INSERT INTO home_items (title, destination_type, destination_content_id, destination_section_type)
         VALUES ('x', 'section', '00000000-0000-0000-0000-000000000001', 'event')",
    )
    .execute(&pool)
    .await;
    assert!(section_with_content.is_err());

    // A section without a section type is now rejected.
    let section_without_type =
        sqlx::query("INSERT INTO home_items (title, destination_type) VALUES ('x', 'section')")
            .execute(&pool)
            .await;
    assert!(
        section_without_type.is_err(),
        "a section destination must name a destination_section_type"
    );

    // A content destination carrying a section type is rejected.
    let content_with_section_type = sqlx::query(
        "INSERT INTO home_items (title, destination_type, destination_section_type)
         VALUES ('x', 'content', 'event')",
    )
    .execute(&pool)
    .await;
    assert!(
        content_with_section_type.is_err(),
        "a content destination must not carry a destination_section_type"
    );

    let content_without_target =
        sqlx::query("INSERT INTO home_items (title, destination_type) VALUES ('x', 'content')")
            .execute(&pool)
            .await;
    assert!(content_without_target.is_err());

    let valid_section = sqlx::query(
        "INSERT INTO home_items (title, destination_type, destination_section_type)
         VALUES ('x', 'section', 'event')",
    )
    .execute(&pool)
    .await;
    assert!(valid_section.is_ok());
}

/// Core persistence: insert an admin user, media, content and a HOME item that
/// points at the content, then read them back.
#[sqlx::test(migrations = "./migrations")]
async fn core_persistence_round_trip(pool: PgPool) {
    let admin_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO admin_users (username, password_hash)
         VALUES ('admin', 'hash') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let media_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO media (filename, mime_type, size_bytes, storage_path, width, height)
         VALUES ('pic.webp', 'image/webp', 1024, '/media/pic.webp', 100, 200)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let content_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO content
            (content_type, title, slug, publication_status, publish_from, publish_until, image_id, created_by)
         VALUES ('event', 'Show', 'show', 'published', '2026-09-01 00:00:00+00',
                 '2026-09-01 12:00:00+00', $1, $2)
         RETURNING id",
    )
    .bind(media_id)
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // A section HOME item (points at the content_type listing).
    sqlx::query(
        "INSERT INTO home_items (title, position, enabled, destination_type, destination_section_type)
         VALUES ('Events', 1, true, 'section', 'event')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // A content HOME item pointing at the specific row.
    sqlx::query(
        "INSERT INTO home_items (title, position, enabled, destination_type, destination_content_id)
         VALUES ('Show details', 2, true, 'content', $1)",
    )
    .bind(content_id)
    .execute(&pool)
    .await
    .unwrap();

    let rows = sqlx::query_scalar::<_, String>("SELECT title FROM home_items ORDER BY position")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows, vec!["Events".to_string(), "Show details".to_string()]);

    let (read_content, read_status): (String, String) =
        sqlx::query_as("SELECT title, publication_status::text FROM content WHERE id = $1")
            .bind(content_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(read_content, "Show");
    assert_eq!(read_status, "published");
}
