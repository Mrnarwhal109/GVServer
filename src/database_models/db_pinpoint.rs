use chrono::{DateTime, Utc};
use uuid::{Uuid};

pub struct DbPinpoint {
    pub pinpoint_id: Uuid,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub added_at: DateTime<Utc>,
    pub contents_id: Uuid,
    pub description: Option<String>,
    pub attachment: Option<Vec<u8>>,
    pub user_id: Uuid,
    pub username: String
}

impl DbPinpoint {}