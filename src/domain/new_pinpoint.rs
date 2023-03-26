pub struct NewPinpoint {
    pub latitude: f64,
    pub longitude: f64,
    pub description: String
}

impl NewPinpoint{
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