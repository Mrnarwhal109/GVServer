#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserSignUp {
    pub email: String,
    pub username: String,
    pub pw: String,
    pub contents_description: Option<String>,
    pub contents_attachment: Option<Vec<u8>>
}