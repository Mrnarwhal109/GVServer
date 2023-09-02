use crate::startup::ApplicationBaseUrl;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
use std::convert::{TryFrom, TryInto};
use argon2::{Argon2, Algorithm, Version, Params};
use argon2::password_hash::SaltString;
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;
use crate::authentication::password;
use crate::domain::user::AppUser;

#[derive(serde::Deserialize)]
pub struct SignUpData {
    username: String,
    password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum SignUpError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SignUpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SignUpError {
    fn status_code(&self) -> StatusCode {
        match self {
            SignUpError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SignUpError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
name = "Adding a new user",
skip(payload, pool),
fields(
subscriber_email = %payload.email,
subscriber_name = %payload.name
)
)]
pub async fn handle_signup(
    payload: web::Json<SignUpData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SignUpError> {
    let new_subscriber = payload.0.try_into().map_err(SignUpError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let subscriber_id = store_new_user(&pool, &mut transaction, )
        .await
        .context("Failed to insert new subscriber in the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(tran, user)
)]
async fn store_new_user(pool: &PgPool, tran: &mut Transaction<'_, Postgres>, user: AppUser) {
    let salt = password::get_salt_string();
    // Match production parameters
    let password_hash = password::compute_password_hash(
        &user.pw, user.salt.expose_secret())?;
    sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash, salt)
            VALUES ($1, $2, $3, $4)",
            self.user_id,
            self.username,
            password_hash,
            salt,
        )
        .execute(pool)
        .await
        .expect("Failed to store test user.");
}


pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
