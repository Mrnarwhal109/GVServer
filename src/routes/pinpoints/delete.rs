use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpResponse, web};
use actix_web::http::header::ContentType;
use sqlx::PgPool;
use crate::domain::Pinpoint;
use crate::domain::TempUsername;
use serde_json;

#[tracing::instrument(
name = "HTTP route : Delete all pinpoints",
skip(pool)
)]
pub async fn handle_delete_all_pinpoints(
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match delete_all_db_pinpoints(&pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "HTTP route : Delete all of a user's pinpoints",
skip(pool, body)
)]
pub async fn handle_delete_all_user_pinpoints(
    pool: web::Data<PgPool>,
    body: web::Json<TempUsername>,
) -> HttpResponse {

    let username: String = match body.0.try_into() {
        Ok(body) => body,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match delete_all_user_db_pinpoints(&pool, &username).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "Deleting all pinpoints in the database",
skip(pool)
)]
pub async fn delete_all_db_pinpoints(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM pinpoints;
        "#
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
            // Using the '?' operator to return early
            // if the function failed, returning a sqlx::Error
            // We will talk about error handling in depth later!
        })?;

    Ok(())
}

#[tracing::instrument(
name = "Deleting all pinpoints in the database",
skip(pool)
)]
pub async fn delete_all_user_db_pinpoints(
    pool: &PgPool,
    username: &String
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM pinpoints WHERE username = $1;
        "#,
        username
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}