use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserResponse {
    pub unique_id: Option<Uuid>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub role_id: Option<i32>,
    pub role_title: Option<String>,
    pub contents_id: Option<Uuid>,
    pub contents_description: Option<String>,
    pub contents_attachment: Option<Vec<u8>>
}