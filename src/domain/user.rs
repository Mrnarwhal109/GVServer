use argon2::password_hash::SaltString;
use secrecy::Secret;

#[derive(Serialize, Deserialize)]
pub struct AppUser {
    pub email: String,
    pub username: String,
    pub pw: Secret<String>,
    pub salt: Secret<SaltString>,
}