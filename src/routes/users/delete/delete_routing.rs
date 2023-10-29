use actix_web::{HttpMessage, HttpRequest, HttpResponse, web};
use sqlx::{PgPool};
use crate::authentication::{AuthPermissions, AuthService};
use crate::routes::users::delete::delete_user_request::DeleteUserRequest;

#[tracing::instrument(
name = "handle_delete_user",
skip(pool, args),
)]
pub async fn handle_delete_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    args: web::Json<DeleteUserRequest>,
) -> HttpResponse {
    let username = args.0.username;
    if username.is_empty() {
        return HttpResponse::BadRequest().finish();
    }
    let req_ext = req.extensions_mut();
    let auth_permissions: &AuthPermissions = req_ext.get::<AuthPermissions>().unwrap();
    if auth_permissions.username != username {
        return HttpResponse::Unauthorized().finish();
    }
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