use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct Pinpoint {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    pub username: String
}

impl Pinpoint {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        username: String
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            username
        }
    }
}

// Conversion spelled out for PinpointData into NewPinpoint
impl TryFrom<PinpointData> for Pinpoint {
    type Error = String;
    fn try_from(value: PinpointData) -> Result<Self, Self::Error> {
        let latitude = value.latitude;
        let longitude = value.longitude;
        let description = value.description;
        let username = value.username;
        Ok(Self { latitude, longitude, description, username })
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
pub struct PinpointData {
    latitude: f64,
    longitude: f64,
    description: String,
    username: String
}

impl PinpointData {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        username: String,
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            username
        }
    }
}