#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DeleteUserRequest {
    pub username: String
}
