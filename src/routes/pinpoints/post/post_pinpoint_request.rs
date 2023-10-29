use std::fmt::{Display, Formatter};
use chrono::Utc;
use uuid::Uuid;
use crate::domain::Pinpoint;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PostPinpointRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    pub attachment: Option<Vec<u8>>,
    pub username: String
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

impl Display for PostPinpointRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lat = &self.latitude;
        let lng = &self.latitude;
        let desc = &self.description;
        let attachment = match &self.attachment {
            None => String::from("NONE"),
            Some(_) => String::from("SOMETHING")
        };
        let usrn = &self.username;
        write!(f, "PostPinpointRequest(lat is {}, lng is {}, desc is {}, attachment is {}, usrn is {}).",
               lat, lng, desc, attachment, usrn)
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