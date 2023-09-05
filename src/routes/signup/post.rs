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
use crate::domain::app_user::AppUser;
use crate::domain::user_email::UserEmail;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignUpData {
    pub email: String,
    pub username: String,
    pub pw: String,
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
skip(payload, pool)
)]
pub async fn handle_signup(
    payload: web::Json<SignUpData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SignUpError> {
    // Salt is generated from initialization
    let new_user = AppUser::try_from(payload.0)
        .map_err(SignUpError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let subscriber_id = store_new_user(&mut transaction, &new_user)
        .await
        .context("Failed to insert new user into the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    Ok(HttpResponse::Ok().finish())
}

async fn store_new_user(
    tran: &mut Transaction<'_, Postgres>,
    user: &AppUser,
) -> Result<Uuid, sqlx::Error> {
    let uuid = user.unique_id;
    let email = user.email.to_string();
    let phash = user.phash.expose_secret().to_string();
    let salt = user.salt.to_string();
    sqlx::query!(
            "INSERT INTO users (id, email, username, phash, salt, role_id)
            VALUES ($1, $2, $3, $4, $5, $6)",
            user.unique_id,
            email,
            user.username,
            phash,
            salt,
            2
        )
        .execute(tran)
        .await?;
    Ok(uuid)
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
