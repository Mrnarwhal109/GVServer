use actix_web::{HttpResponse, web};
use sqlx::{PgPool};
use crate::authentication::{AuthParameters, AuthPermissions, AuthService};
use crate::domain::app_user::DeleteUserRequest;

#[tracing::instrument(
name = "handle_delete_user",
skip(pool, auth, args, auth_params),
)]
pub async fn handle_delete_user(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    args: web::Json<DeleteUserRequest>,
    auth_params: AuthParameters,
) -> HttpResponse {
    let permissions: AuthPermissions;
    let username = args.0.username;
    if username.is_empty() {
        return HttpResponse::BadRequest().finish();
    }
    match auth.validate_request_for_user(
        &auth_params, username.clone()) {
        Ok(x) => permissions = x,
        Err(_) => {
            return HttpResponse::Unauthorized().finish();
        }
    };

    match delete_db_user(&pool, &username).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

pub async fn delete_db_user(
    pool: &PgPool,
    username: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM users
        WHERE username = $1;
        "#
        , username
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

   Ok(())
}