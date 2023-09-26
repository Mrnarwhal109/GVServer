use uuid::Uuid;
use actix_web::{HttpResponse, web};
use sqlx::{PgPool, Postgres, Transaction};
use crate::authentication::{AuthParameters, AuthService};
use crate::domain::pinpoint::DeletePinpointRequest;

#[tracing::instrument(
name = "handle_delete_pinpoint",
skip(pool, auth, auth_params),
)]
pub async fn handle_delete_pinpoints(
    args: web::Json<DeletePinpointRequest>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    auth_params: AuthParameters
) -> HttpResponse {
    let username = args.0.username.unwrap_or(String::from(""));
    if username.is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    match auth.validate_request_for_user(
        &auth_params, username.clone()) {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    let mut tran = match pool.begin().await {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    match args.0.pinpoint_id {
        None => {
            match delete_user_db_pinpoints(&mut tran, &username).await {
                Ok(_) =>
                    {
                        match tran.commit().await {
                            Ok(_) => {
                                HttpResponse::Ok().finish()
                            }
                            Err(_) => {
                                HttpResponse::InternalServerError().finish()
                            }
                        }
                    },
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        }
        Some(x) => {
            let response = match delete_db_pinpoint(&mut tran, x).await {
                Ok(_) =>
                    {
                        match tran.commit().await {
                            Ok(_) => {
                                HttpResponse::Ok().finish()
                            }
                            Err(_) => {
                                HttpResponse::InternalServerError().finish()
                            }
                        }
                    },
                Err(_) => HttpResponse::InternalServerError().finish()
            };
            response
        }
    }
}

pub async fn delete_db_pinpoint(
    tran: &mut Transaction<'_, Postgres>,
    pinpoint_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM pinpoint_contents
        WHERE pinpoint_id = $1;
        "#
        , pinpoint_id
    )
        .execute(&mut *tran)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
            // Using the '?' operator to return early
            // if the function failed, returning a sqlx::Error
        })?;

    sqlx::query!(
        r#"
        DELETE FROM pinpoints
        WHERE id = $1;
        "#
        , pinpoint_id
    )
        .execute(&mut *tran)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
            // Using the '?' operator to return early
            // if the function failed, returning a sqlx::Error
        })?;

    Ok(())
}

pub async fn delete_user_db_pinpoints(
    tran: &mut Transaction<'_, Postgres>,
    username: &String
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH usr_pin(pinpoint_id) AS
        (
            SELECT pinpoint_id
            FROM user_pinpoints
            WHERE user_id IN (SELECT id FROM users WHERE username = $1)
        )
        DELETE FROM pinpoints
        WHERE id IN (SELECT pinpoint_id FROM usr_pin);
        "#
        , username
    )
        .execute(tran)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
            // Using the '?' operator to return early
            // if the function failed, returning a sqlx::Error
        })?;

    Ok(())
}