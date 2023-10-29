#[derive(serde::Serialize, serde::Deserialize)]
pub struct PostUserRequest {
    pub email: String,
}