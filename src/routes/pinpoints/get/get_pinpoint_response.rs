use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::domain::Pinpoint;
use chrono::serde::ts_seconds;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetPinpointResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    #[serde(with = "ts_seconds")]
    pub added_at: DateTime<Utc>,
    pub attachment: Vec<u8>,
    pub pinpoint_id: Option<Uuid>,
    pub pinpoint_user_id: Option<Uuid>,
    pub pinpoint_username: Option<String>,
}

impl GetPinpointResponse {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        added_at: DateTime<Utc>,
        attachment: Vec<u8>,
        pinpoint_id: Option<Uuid>,
        pinpoint_user_id: Option<Uuid>,
        pinpoint_username: Option<String>
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            added_at,
            attachment,
            pinpoint_id,
            pinpoint_user_id,
            pinpoint_username
        }
    }

    pub fn clone_as_censored(&self) -> Self {
        let cloned = Self::new(
            self.latitude, self.longitude, self.description.clone(),
            self.added_at.clone(), self.attachment.clone(),
            None, None, None);
        cloned
    }
}

impl Clone for GetPinpointResponse {
    fn clone(&self) -> Self {
        let cloned = Self::new(
            self.latitude, self.longitude, self.description.clone(),
            self.added_at.clone(), self.attachment.clone(),
            self.pinpoint_id.clone(), self.pinpoint_user_id.clone(),
            self.pinpoint_username.clone());
        cloned
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
            pinpoint_id: Some(value.pinpoint_id.clone()),
            pinpoint_user_id: value.user_id,
            pinpoint_username: Some(value.username.clone()) })
    }
}