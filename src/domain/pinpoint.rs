use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use chrono::serde::ts_seconds;
use uuid::Uuid;
use crate::database_models::DbPinpoint;

pub struct Pinpoint {
    pub pinpoint_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub added_at: DateTime<Utc>,
    pub contents_id: Uuid,
    pub description: String,
    pub attachment: Option<Vec<u8>>,
    pub user_id: Option<Uuid>,
    pub username: String
}

impl Pinpoint {
    pub fn new(
        pinpoint_id: Uuid,
        latitude: f64,
        longitude: f64,
        added_at: DateTime<Utc>,
        contents_id: Uuid,
        description: String,
        attachment: Option<Vec<u8>>,
        user_id: Option<Uuid>,
        username: String
    ) -> Self {
        Self {
            pinpoint_id,
            latitude,
            longitude,
            added_at,
            contents_id,
            description,
            attachment,
            user_id,
            username
        }
    }
}

// Conversion spelled out for DbPinpoint into Pinpoint
impl TryFrom<&DbPinpoint> for Pinpoint {
    type Error = String;
    fn try_from(value: &DbPinpoint) -> Result<Self, Self::Error> {
        let pinpoint_id = value.pinpoint_id;
        let latitude = value.latitude.unwrap_or(0.0);
        let longitude = value.longitude.unwrap_or(0.0);
        let added_at = value.added_at;
        let contents_id = value.contents_id;
        let description = value.description.clone().unwrap_or(String::from(""));
        let attachment = value.attachment.clone();
        let user_id = value.user_id;
        let username = value.username.clone();
        Ok(Self { pinpoint_id, latitude, longitude, added_at, contents_id,
            description, attachment, username, user_id: Some(user_id) })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TempUsername {
    username: String,
}

impl TryFrom<TempUsername> for String {
    type Error = String;
    fn try_from(value: TempUsername) -> Result<Self, Self::Error> {
        Ok(value.username.to_string())
    }
}