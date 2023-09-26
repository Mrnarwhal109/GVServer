use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use crate::domain::Pinpoint;
use crate::domain::PostPinpointRequest;
use crate::authentication::{AuthParameters, AuthService};

#[tracing::instrument(
name = "handle_add_pinpoint",
skip(pool, auth, auth_params),
)]
pub async fn handle_add_pinpoint(
    pinpoint: web::Json<PostPinpointRequest>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    auth_params: AuthParameters
) -> HttpResponse {
    // 'web::Json' is a wrapper around 'PinpointRequest'
    // 'pinpoint.0' gives us access to the underlying 'PinpointRequest'
    // You can use e.g. Pinpoint::try_from(pinpoint.0);
    let new_pinpoint: Pinpoint = match pinpoint.0.try_into() {
        Ok(pinpoint) => pinpoint,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match auth.validate_request_for_user(
        &auth_params, new_pinpoint.username.clone()) {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

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
            // if the function failed, returning a sqlx::Error
            // We will talk about error handling in depth later!
        })?;


    Ok(())
}