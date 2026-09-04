use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown value `{0}`")]
pub struct UnknownEnumValue(pub String);

/// Content types supported by the MVP (REQUIREMENTS FR-004).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Type,
)]
#[sqlx(type_name = "content_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Page,
    News,
    Event,
    Place,
    Contact,
    Gallery,
}

impl ContentType {
    pub fn as_str(self) -> &'static str {
        match self {
            ContentType::Page => "page",
            ContentType::News => "news",
            ContentType::Event => "event",
            ContentType::Place => "place",
            ContentType::Contact => "contact",
            ContentType::Gallery => "gallery",
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ContentType {
    type Err = UnknownEnumValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "page" => Ok(ContentType::Page),
            "news" => Ok(ContentType::News),
            "event" => Ok(ContentType::Event),
            "place" => Ok(ContentType::Place),
            "contact" => Ok(ContentType::Contact),
            "gallery" => Ok(ContentType::Gallery),
            _ => Err(UnknownEnumValue(s.to_string())),
        }
    }
}

/// Publication state of content (REQUIREMENTS FR-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Type)]
#[sqlx(type_name = "publication_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PublicationStatus {
    #[default]
    Draft,
    Published,
    Archived,
}

impl PublicationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PublicationStatus::Draft => "draft",
            PublicationStatus::Published => "published",
            PublicationStatus::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PublicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PublicationStatus {
    type Err = UnknownEnumValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(PublicationStatus::Draft),
            "published" => Ok(PublicationStatus::Published),
            "archived" => Ok(PublicationStatus::Archived),
            _ => Err(UnknownEnumValue(s.to_string())),
        }
    }
}

/// Destination of a HOME button (REQUIREMENTS FR-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(type_name = "home_destination_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HomeDestinationType {
    Section,
    Content,
}

impl HomeDestinationType {
    pub fn as_str(self) -> &'static str {
        match self {
            HomeDestinationType::Section => "section",
            HomeDestinationType::Content => "content",
        }
    }
}

impl std::fmt::Display for HomeDestinationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HomeDestinationType {
    type Err = UnknownEnumValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "section" => Ok(HomeDestinationType::Section),
            "content" => Ok(HomeDestinationType::Content),
            _ => Err(UnknownEnumValue(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Media {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub id: Uuid,
    pub content_type: ContentType,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub body: Option<String>,
    pub publication_status: PublicationStatus,
    pub publish_from: Option<chrono::DateTime<chrono::Utc>>,
    pub publish_until: Option<chrono::DateTime<chrono::Utc>>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub image_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeItem {
    pub id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub image_id: Option<Uuid>,
    pub position: i32,
    pub enabled: bool,
    pub destination_type: HomeDestinationType,
    pub destination_content_id: Option<Uuid>,
}

impl AdminUser {
    pub fn new(id: Uuid, username: String, is_active: bool) -> Self {
        Self {
            id,
            username,
            is_active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type_round_trips() {
        for value in [
            ContentType::Page,
            ContentType::News,
            ContentType::Event,
            ContentType::Place,
            ContentType::Contact,
            ContentType::Gallery,
        ] {
            assert_eq!(ContentType::from_str(value.as_str()).unwrap(), value);
        }
        assert!(matches!(
            ContentType::from_str("bogus"),
            Err(UnknownEnumValue(_))
        ));
    }

    #[test]
    fn publication_status_round_trips() {
        for value in [
            PublicationStatus::Draft,
            PublicationStatus::Published,
            PublicationStatus::Archived,
        ] {
            assert_eq!(PublicationStatus::from_str(value.as_str()).unwrap(), value);
        }
        assert!(matches!(
            PublicationStatus::from_str("pending"),
            Err(UnknownEnumValue(_))
        ));
    }

    #[test]
    fn home_destination_round_trips() {
        for value in [HomeDestinationType::Section, HomeDestinationType::Content] {
            assert_eq!(
                HomeDestinationType::from_str(value.as_str()).unwrap(),
                value
            );
        }
        assert!(matches!(
            HomeDestinationType::from_str("nope"),
            Err(UnknownEnumValue(_))
        ));
    }
}
