use std::fmt::{Display, Formatter};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum AuthPermissionsMode {
    None,
    Restrict,
    Allow,
    MAX
}

impl Display for AuthPermissionsMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "test")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthPermissions {
    pub mode: AuthPermissionsMode,
    pub username: String,
}

impl AuthPermissions {
    pub fn new(
        mode: AuthPermissionsMode,
        username: String
    ) -> Self {
        Self { mode, username }
    }
}