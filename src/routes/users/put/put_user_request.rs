#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PutUserRequest {
    // The username that it should be changed to
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub contents_description: Option<String>,
    pub contents_attachment: Option<Vec<u8>>
}

impl PutUserRequest {
    pub fn is_empty(&self) -> bool {
        return self.username.is_none()
        && self.email.is_none()
        && self.password.is_none()
        && self.contents_description.is_none()
        && self.contents_attachment.is_none()
    }
}