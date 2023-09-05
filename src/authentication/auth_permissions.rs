#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum AuthPermissionsMode {
    None,
    WriteForUniqueUser,
    WriteForAll,
    MAX
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthPermissions {
    mode: AuthPermissionsMode
}

impl AuthPermissions {
    pub fn new(mode: AuthPermissionsMode) -> Self {
        Self { mode }
    }
}