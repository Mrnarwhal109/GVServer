use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpResponse, web};
use actix_web::http::header::ContentType;
use sqlx::PgPool;
use crate::domain::Pinpoint;
use crate::domain::PinpointData;
use serde_json;

#[tracing::instrument(
name = "Adding a new pinpoint",
skip(pinpoint, pool)
)]
pub async fn handle_add_pinpoint(
    pinpoint: web::Json<PinpointData>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // 'web::Form' is a wrapper around 'FormData'
    // 'form.0' gives us access to the underlying 'FormData'
    // You can use NewSubscriber::try_from(form.0);
    let new_pinpoint = match pinpoint.0.try_into() {
        Ok(pinpoint) => pinpoint,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_pinpoint(&pool, &new_pinpoint).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "Saving new pinpoint details in the database",
skip(new_pinpoint, pool)
)]
pub async fn insert_pinpoint(
    pool: &PgPool,
    new_pinpoint: &Pinpoint,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO pinpoints (id, latitude, longitude, description, username, added_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        Uuid::new_v4(),
        new_pinpoint.latitude,
        new_pinpoint.longitude,
        new_pinpoint.description,
        new_pinpoint.username,
        Utc::now()
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