use uuid::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct DbUser {
    pub unique_id: Uuid,
    pub email: String,
    pub username: String,
    pub phash: String,
    pub salt: String,
    pub role_id: i32,
    pub role_title: String,
    pub contents_id: Option<Uuid>,
    pub contents_description: Option<String>,
    pub contents_attachment: Option<Vec<u8>>
}

impl DbUser {}