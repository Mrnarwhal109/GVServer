use argon2::password_hash::SaltString;
use secrecy::Secret;
use uuid::Uuid;
use crate::authentication::{compute_password_hash, rand_salt_string};
use crate::domain::user_email::UserEmail;
use crate::routes::users::post::{UserSignUp, SignUpError};

pub struct AppUser {
    pub unique_id: Uuid,
    pub email: UserEmail,
    pub username: String,
    pub phash: Secret<String>,
    pub salt: SaltString,
    pub role_id: i32,
    pub role_title: String,
}

impl TryFrom<UserSignUp> for AppUser {
    type Error = SignUpError;

    fn try_from(value: UserSignUp) -> Result<Self, Self::Error> {
        let unique_id: Uuid = Uuid::new_v4();
        let email: UserEmail = UserEmail::parse(value.email)
            .map_err(SignUpError::ValidationError)?;
        let pw: Secret<String> = Secret::new(value.pw);
        let username = value.username;
        let salt = rand_salt_string();

        let phash = compute_password_hash(&pw, &salt)
            .map_err(|e| SignUpError::ValidationError(e.to_string()))?;

        Ok(Self{unique_id, email, username, phash, salt,
            role_id: -1, role_title: String::from("UNKNOWN") })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DeleteUserRequest {
    pub username: String
}
