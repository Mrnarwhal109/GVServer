use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserResponse {
    pub unique_id: Option<Uuid>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub role_id: Option<i32>,
    pub role_title: Option<String>
}