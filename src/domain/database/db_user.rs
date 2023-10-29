use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DbUser {
    pub unique_id: Uuid,
    pub email: String,
    pub username: String,
    pub phash: String,
    pub salt: String,
    pub role_id: i32,
    pub role_title: String,
}