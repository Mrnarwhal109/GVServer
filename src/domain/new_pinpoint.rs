use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct Pinpoint {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String
}

impl Pinpoint {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String
    ) -> Self {
        Self {
            latitude,
            longitude,
            description
        }
    }
}