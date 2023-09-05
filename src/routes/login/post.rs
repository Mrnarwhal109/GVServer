use crate::authentication::AuthError;
use crate::authentication::{validate_credentials, Credentials};
use actix_web::error::InternalError;
use actix_web::web;
use actix_web::HttpResponse;
use secrecy::Secret;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct LoginData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
skip(content, pool),
fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
// We are now injecting `PgPool` to retrieve stored credentials from the database
pub async fn handle_login(
    content: web::Json<LoginData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let credentials = Credentials {
        username: content.0.username,
        password: content.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    Ok(HttpResponse::Ok().finish())
}