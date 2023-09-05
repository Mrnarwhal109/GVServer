use argon2::password_hash::SaltString;
use secrecy::Secret;
use serde::Serialize;
use serde::Deserialize;
use serde_json::from_str;
use uuid::Uuid;
use crate::authentication::{compute_password_hash, get_salt_string};
use crate::domain::user_email::UserEmail;
use crate::routes::post::SignUpData;

pub struct AppUser {
    pub unique_id: Uuid,
    pub email: UserEmail,
    pub username: String,
    pub phash: Secret<String>,
    pub salt: SaltString,
}

impl TryFrom<SignUpData> for AppUser {
    type Error = String;

    fn try_from(value: SignUpData) -> Result<Self, Self::Error> {
        let unique_id: Uuid = uuid::Uuid::new_v4();
        let email: UserEmail = UserEmail::parse(value.email)?;
        let pw: Secret<String> = Secret::new(value.pw);
        let username = value.username;
        let salt = get_salt_string();
        let phash = match compute_password_hash(&pw, &salt) {
            Ok(hash) => hash,
            Err(e) =>
                {
                    println!("Error computing password hash!");
                    return Err(Self::Error::new())
                }
        };
        Ok(Self{unique_id, email, username, phash, salt})
    }
}