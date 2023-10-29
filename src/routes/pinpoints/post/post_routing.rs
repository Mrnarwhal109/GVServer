use std::string::FromUtf8Error;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use sqlx::PgPool;
use crate::domain::Pinpoint;
use crate::authentication::{AuthParameters, AuthPermissions, AuthService};
use crate::routes::pinpoints::post::post_pinpoint_request::PostPinpointRequest;

#[tracing::instrument(
name = "handle_add_pinpoint",
skip(pool),
)]
pub async fn handle_add_pinpoint(
    req: HttpRequest,
    pinpoint: web::Json<PostPinpointRequest>,
    // Retrieving a connection from the application state
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // 'web::Json' is a wrapper around 'PostPinpointRequest'
    // 'pinpoint.0' gives us access to the underlying 'PostPinpointRequest'
    // You can use e.g. PostPinpointRequest::try_from(pinpoint.0);
    let new_pinpoint: Pinpoint = match pinpoint.0.try_into() {
        Ok(pinpoint) => pinpoint,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let req_ext = req.extensions_mut();
    let auth_permissions: &AuthPermissions = req_ext.get::<AuthPermissions>().unwrap();
    println!("AuthPermissions found as {:?}", auth_permissions);
    if auth_permissions.username != new_pinpoint.username {
        return HttpResponse::Unauthorized().finish();
    }
    match insert_pinpoint(&pool, &new_pinpoint).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

pub async fn insert_pinpoint(
    pool: &PgPool,
    new_pinpoint: &Pinpoint,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
WITH pin AS (
INSERT INTO pinpoints (id, latitude, longitude)
VALUES ($1, $2, $3)
RETURNING id
),
con as (
    INSERT INTO contents (id, description, attachment)
    VALUES($4, $5, $6)
    RETURNING id
),
usr_pin as (
    INSERT INTO user_pinpoints (pinpoint_id, user_id)
    SELECT id, (SELECT id FROM users WHERE username = $7) FROM pin
)
INSERT INTO pinpoint_contents (pinpoint_id, content_id)
SELECT pin.id, con.id FROM pin, con
        "#,
        new_pinpoint.pinpoint_id,
        new_pinpoint.latitude,
        new_pinpoint.longitude,
        new_pinpoint.contents_id,
        new_pinpoint.description,
        new_pinpoint.attachment,
        new_pinpoint.username
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
            // Using the '?' operator to return early
            // if the function failed, returning a sqlx::Error.
        })?;
    Ok(())
}