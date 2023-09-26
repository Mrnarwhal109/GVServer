use chrono::{DateTime, Utc};
use sqlx::{Error, FromRow};
use uuid::{Uuid};

#[derive(sqlx::FromRow)]
pub struct DbPinpoint {
    #[sqlx]
    pub pinpoint_id: Uuid,
    #[sqlx]
    pub latitude: Option<f64>,
    #[sqlx]
    pub longitude: Option<f64>,
    #[sqlx]
    pub added_at: DateTime<Utc>,
    #[sqlx]
    pub contents_id: Uuid,
    #[sqlx]
    pub description: Option<String>,
    #[sqlx]
    pub attachment: Option<Vec<u8>>,
    #[sqlx]
    pub user_id: Uuid,
    #[sqlx]
    pub username: String
}

impl DbPinpoint {}