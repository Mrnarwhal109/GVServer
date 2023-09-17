use std::vec;
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

// Conversion spelled out for PinpointData into Pinpoint
impl TryFrom<PostPinpointRequest> for Pinpoint {
    type Error = String;
    fn try_from(value: PostPinpointRequest) -> Result<Self, Self::Error> {
        let pinpoint_id = Uuid::new_v4();
        let contents_id = Uuid::new_v4();
        let user_id = None;
        let attachment = value.attachment;
        let latitude = value.latitude;
        let longitude = value.longitude;
        let description = value.description;
        let username = value.username;
        let added_at = Utc::now();
        Ok(Self { pinpoint_id, latitude, longitude, added_at, contents_id,
            description, attachment, username, user_id })
    }
}

// Conversion spelled out for PinpointData into Pinpoint
impl TryFrom<&Pinpoint> for GetPinpointResponse {
    type Error = String;
    fn try_from(value: &Pinpoint) -> Result<Self, Self::Error> {
        let attachment = value.attachment.clone().unwrap_or(Vec::new());
        let latitude = value.latitude;
        let longitude = value.longitude;
        let description = value.description.clone();
        let added_at = Utc::now();
        Ok(Self { latitude, longitude, added_at,
            description, attachment,
            pinpoint_user_id: value.user_id,
            pinpoint_username: Some(value.username.clone()) })
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostPinpointRequest {
    latitude: f64,
    longitude: f64,
    description: String,
    attachment: Option<Vec<u8>>,
    username: String
}

impl PostPinpointRequest {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        attachment: Option<Vec<u8>>,
        username: String
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            attachment,
            username
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetPinpointRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius: Option<f64>,
    pub pinpoint_id: Option<Uuid>,
    pub username: Option<String>
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetPinpointResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    #[serde(with = "ts_seconds")]
    pub added_at: DateTime<Utc>,
    pub attachment: Vec<u8>,
    pub pinpoint_user_id: Option<Uuid>,
    pub pinpoint_username: Option<String>
}

impl GetPinpointResponse {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        added_at: DateTime<Utc>,
        attachment: Vec<u8>,
        pinpoint_user_id: Option<Uuid>,
        pinpoint_username: Option<String>
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            added_at,
            attachment,
            pinpoint_user_id,
            pinpoint_username
        }
    }

    pub fn clone_as_censored(&self) -> Self {
        let cloned = Self::new(
            self.latitude, self.longitude, self.description.clone(),
            self.added_at.clone(), self.attachment.clone(),
            None, None);
        cloned
    }
}

impl Clone for GetPinpointResponse {
    fn clone(&self) -> Self {
        let cloned = Self::new(
            self.latitude, self.longitude, self.description.clone(),
            self.added_at.clone(), self.attachment.clone(),
            self.pinpoint_user_id.clone(), self.pinpoint_username.clone());
        cloned
    }
}