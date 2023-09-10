use std::fmt::{Display, Formatter};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum AuthPermissionsMode {
    None,
    WriteForUniqueUser,
    WriteForAll,
    MAX
}

impl Display for AuthPermissionsMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "test")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthPermissions {
    pub mode: AuthPermissionsMode
}

impl AuthPermissions {
    pub fn new(mode: AuthPermissionsMode) -> Self {
        Self { mode }
    }
}