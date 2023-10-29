#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserSignUp {
    pub email: String,
    pub username: String,
    pub pw: String,
}