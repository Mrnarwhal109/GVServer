use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse, ResponseError, HttpRequest};
use anyhow::{Context};
use sqlx::{PgPool, Postgres, Transaction};
use std::convert::{TryFrom};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;
use crate::authentication::basic_authentication;
use crate::domain::app_user::AppUser;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SignUpData {
    pub email: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserSignUp {
    pub email: String,
    pub username: String,
    pub pw: String,
}

// Using .map_err(SignUpError::ChoiceType)?;
// required the try_from function called before map_err
// to choose the type Error of the type inside the enum choice,
// a.k.a. String for ValidationError(String).
#[derive(thiserror::Error)]
pub enum SignUpError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
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
    request: HttpRequest,
    payload: web::Json<SignUpData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let credentials = match basic_authentication(&request.headers()) {
        Ok(c) => c,
        Err(e) => {
            println!("Failure from basic auth. {}", e.to_string());
            return HttpResponse::BadRequest().finish()
        }
    };
    let email = payload.0.email;
    let combined_payload = UserSignUp {
        email,
        username: credentials.username,
        pw: credentials.pw.expose_secret().to_string(),
    };
    match sign_up_user(combined_payload, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => {
            println!("Failure from sign_up_user.");
            HttpResponse::BadRequest().finish()
        }
    }
}

pub async fn sign_up_user(
    user_sign_up: UserSignUp,
    pool: &PgPool,
) -> Result<(), anyhow::Error> {
    // Salt and hashing are generated within try_from
    let new_user = AppUser::try_from(user_sign_up)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    store_new_user(&mut transaction, &new_user)
        .await
        .context("Failed to insert new user into the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    Ok(())
}

async fn store_new_user(
    tran: &mut Transaction<'_, Postgres>,
    user: &AppUser,
) -> Result<Uuid, sqlx::Error> {
    let uuid = user.unique_id;
    let email = user.email.to_string();
    let phash = user.phash.expose_secret().to_string();
    let salt = user.salt.to_string();
    let execute_result = sqlx::query!(
            r#"WITH usr AS
            (INSERT INTO users (id, email, username, phash, salt)
            VALUES ($1, $2, $3, $4, $5) RETURNING id)
            INSERT INTO user_roles (user_id, role_id)
            SELECT id, $6 FROM usr; "#,
            user.unique_id,
            email,
            user.username,
            phash,
            salt,
            2
        )
        .execute(tran)
        .await?;
    let rows_hit = execute_result.rows_affected().to_string();
    println!("Rows hit: {}", rows_hit);
    println!("Db user ID stored: {}", uuid);
    println!("Db phash stored: {}", phash.to_string());
    println!("Db salt stored: {}", salt.to_string());
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
