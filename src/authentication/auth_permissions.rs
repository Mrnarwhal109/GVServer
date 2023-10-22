use std::fmt::{Display, Formatter};
use actix_web::{dev, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use crate::authentication::AuthParameters;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum AuthPermissionsMode {
    None,
    Restrict,
    Allow,
    MAX
}

impl Display for AuthPermissionsMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "AuthPermissionsModeValue")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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