use uuid::Uuid;
use crate::authentication::{rand_salt_string};

pub struct DbUser {
    pub unique_id: Uuid,
    pub email: String,
    pub username: String,
    pub phash: String,
    pub salt: String,
    pub role_id: i32,
    pub role_title: String,
}

impl DbUser {}