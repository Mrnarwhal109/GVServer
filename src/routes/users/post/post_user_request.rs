#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PostUserRequest {
    pub email: String,
    pub contents_description: Option<String>,
    pub contents_attachment: Option<Vec<u8>>
}