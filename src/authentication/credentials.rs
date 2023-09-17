use actix_web::http::header::HeaderMap;
use crate::telemetry::spawn_blocking_with_tracing;
use anyhow::{anyhow, Context};
use argon2::password_hash::{SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, Version};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use base64;

use base64::{Engine as _, alphabet, engine::{self, general_purpose}};

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);


#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("{0}")]
    InvalidCredentials(String),
    #[error("{0}")]
    UnexpectedError(String),
}

impl From<anyhow::Error> for AuthError {
    fn from(value: anyhow::Error) -> Self {
        AuthError::UnexpectedError(value.to_string())
    }
}

pub struct Credentials {
    pub username: String,
    pub pw: Secret<String>,
    pub salt: Option<SaltString>,
}

impl Clone for Credentials {
    fn clone(&self) -> Self {
        Credentials {
            username: self.username.to_string(),
            pw: Secret::new(self.pw.expose_secret().to_string()),
            salt: self.salt.clone()
        }
    }
}

#[tracing::instrument(name = "Get stored credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>, String)>, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT u.id, u.phash, u.salt
        FROM users u
        WHERE u.username = $1
        "#,
        username,
    )
        .fetch_optional(pool)
        .await
        .context("Failed to perform a query to retrieve stored credentials.")?
        .map(|row| (row.id, Secret::new(row.phash), row.salt));
    Ok(row)
}

#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, AuthError> {
    let mut user_id = None;
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );
    let expected_salt = SaltString::new(
        "SaltySaltySaltySaltySalty"
    );
    let mut expected_salt = expected_salt.map_err(|_| anyhow!("Salt invalid"))?;

    if let Some((stored_user_id, stored_password_hash, stored_salt)) =
        get_stored_credentials(&credentials.username, pool).await
            .map_err(|_e| ->
                          AuthError {
                AuthError::UnexpectedError(String::from("Stored credentials failure"))
            }
                    )?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_password_hash;
        expected_salt = SaltString::new(&stored_salt).map_err(|_| anyhow!("Salt invalid"))?;
        println!("Db user ID retrieved: {}", stored_user_id);
        println!("Db phash retrieved: {}", expected_password_hash.expose_secret().to_string());
        println!("Db salt retrieved: {}", expected_salt.to_string());
    }
    let t = spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_password_hash, credentials.pw,
                             expected_salt)
    }).await.map_err(|_| AuthError::UnexpectedError(String::from("validate_credentials(...) failed.")))?;

    match t {
        Ok(_) => {
            println!("spawn_blocking_with_tracing(...) succeeded.")
        }
        Err(_) => {
            println!("spawn_blocking_with_tracing(...) did not succeed.");
            return Err(AuthError::UnexpectedError(String::from("spawn_blocking")));
        }
    };

    user_id
        .ok_or_else(|| AuthError::InvalidCredentials(String::from("Unknown username.")))
}

#[tracing::instrument(
name = "Validate credentials",
skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
    salt_candidate: SaltString
) -> Result<(), AuthError> {
    let hash_found =
        compute_password_hash(&password_candidate, &salt_candidate)?;
    let expected_password_hash = PasswordHash::new(
        expected_password_hash.expose_secret(),
    )
        .map_err(|e| AuthError::InvalidCredentials(e.to_string()))?
        .to_string();

    match hash_found.expose_secret().to_string() == expected_password_hash {
        true => {
            println!("Credentials matched!");
            Ok(())
        },
        false => {
            println!("Credentials did not match!");
            Err(AuthError::InvalidCredentials(String::from("Invalid credentials")))
        }
    }
}

pub fn rand_salt_string() -> SaltString {
    let salt = SaltString::generate(&mut rand::thread_rng());
    salt
}

pub fn compute_password_hash(password: &Secret<String>, salt: &SaltString)
    -> Result<Secret<String>, AuthError> {
    // Match production parameters
    let password_hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
        .hash_password(password.expose_secret().as_bytes(), salt)
        .map_err(|_| AuthError::UnexpectedError(String::from("Hashing failure")))?
        .to_string();
    Ok(Secret::new(password_hash))
}

pub fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
// The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_credentials = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_bytes =
        general_purpose::STANDARD.decode( base64encoded_credentials)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?;
// Split into two segments, using ':' as delimiter
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A username must be provided in 'Basic' auth.")
        })?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A password must be provided in 'Basic' auth.")
        })?
        .to_string();

    println!("Basic auth decoded as username {} pw {}", username.clone(), password.clone());
    Ok(Credentials {
        username,
        pw: Secret::new(password),
        salt: None
    })
}