use uuid::Uuid;
use crate::authentication::{compute_password_hash, get_salt_string};
use crate::domain::user_email::UserEmail;
use crate::routes::post::SignUpData;

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