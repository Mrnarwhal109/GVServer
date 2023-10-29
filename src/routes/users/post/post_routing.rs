use actix_web::http::{StatusCode};
use actix_web::{post, web, HttpResponse, HttpRequest};
use anyhow::{anyhow};
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};
use std::convert::{TryFrom};
use secrecy::{ExposeSecret};
use uuid::Uuid;
use crate::authentication::{AuthParameters, AuthService, basic_authentication, validate_credentials};
use crate::domain::app_user::AppUser;
use crate::domain::user_sign_up::UserSignUp;
use crate::routes::users::post::post_user_request::PostUserRequest;

#[tracing::instrument(
name = "handle_signup",
skip(payload, pool, auth)
)]
pub async fn handle_signup(
    request: HttpRequest,
    payload: web::Json<PostUserRequest>,
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
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
        username: credentials.username.clone(),
        pw: credentials.pw.expose_secret().to_string(),
        contents_description: payload.0.contents_description,
        contents_attachment: payload.0.contents_attachment
    };
    let mut transaction: Transaction<Postgres> = match pool.begin().await {
        Ok(x) => x,
        Err(_) => {
            println!("Failed to acquire a Postgres connection from the pool");
            return HttpResponse::InternalServerError().finish()
        }
    };
    match sign_up_user(combined_payload, &mut transaction).await {
        Ok(_) => {},
        Err(_) => {
            println!("Failure from sign_up_user.");
            match transaction.rollback().await { Ok(_) | Err(_) => {} };
            return HttpResponse::BadRequest().finish()
        }
    }
    match transaction.commit().await {
        Ok(_) => {}
        Err(_) => {
            println!("Failed to commit transaction.");
            return HttpResponse::InternalServerError().finish()
        }
    }
    match validate_credentials(credentials.clone(), &pool).await {
        Ok(_) => {
            let auth_jwt = auth.create_jwt(
                credentials.clone().username.to_string().as_str()).await;
            let auth_json = serde_json::json!({
                "jwt": auth_jwt
                });
            let good_response = HttpResponse::build(StatusCode::OK)
                .json(auth_json);
            good_response
        },
        Err(_) => {
            HttpResponse::BadRequest().finish()
        }
    }
}

pub async fn sign_up_user(
    user_sign_up: UserSignUp,
    tran: &mut Transaction<'_, Postgres>,
) -> Result<(), anyhow::Error> {
    // Salt and hashing are generated within try_from
    let new_user = AppUser::try_from(user_sign_up)?;
    match store_new_user(tran, &new_user).await {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("Failed to store new user in the database.");
            Err(anyhow!("Failed to store new user in the database."))
        }
    }
}

async fn store_new_user(
    tran: &mut Transaction<'_, Postgres>,
    user: &AppUser,
) -> Result<Uuid, sqlx::Error> {
    let uuid = user.unique_id;
    let email = user.email.to_string();
    let phash = user.phash.expose_secret().to_string();
    let salt = user.salt.to_string();
    // Voodoo dealing with re-borrowing
    let initial_ref = &mut *tran;
    {
        sqlx::query!(
            r#"
            WITH usr AS (
                INSERT INTO users (id, email, username, phash, salt)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id
            )
            INSERT INTO user_roles (user_id, role_id)
            (SELECT id, $6 FROM usr);
            "#,
            user.unique_id,
            email,
            user.username,
            phash,
            salt,
            2
        )
            .execute(initial_ref)
            .await?;
    }
    {
        // Voodoo dealing with re-borrowing
        let another = &mut *tran;
        if user.contents_id.is_some() {
            let attempt = store_new_user_contents(another, user).await?;
        }
    }
    /*
    let rows_hit = execute_result.rows_affected().to_string();
    println!("Rows hit: {}", rows_hit);
    println!("Db user ID stored: {}", uuid);
    println!("Db phash stored: {}", phash.to_string());
    println!("Db salt stored: {}", salt.to_string());
     */
    Ok(uuid)
}

async fn store_new_user_contents(
    tran: &mut Transaction<'_, Postgres>,
    user: &AppUser,
) -> Result<Uuid, sqlx::Error> {
    let execute_result = sqlx::query!(
            r#"
            WITH cts AS (
                INSERT INTO contents (id, description, attachment)
                VALUES ($1, $2, $3)
                RETURNING id
            )
            INSERT INTO user_contents (user_id, contents_id)
            (SELECT $4, id FROM cts);
            "#,
            user.contents_id,
            user.contents_description,
            user.contents_attachment,
            user.unique_id
        )
        .execute(tran)
        .await?;
    Ok(user.unique_id)
}

#[tracing::instrument(
name = "handle_modify_user",
skip(pool, auth, auth_params),
)]
#[post("/{username}")]
pub async fn handle_modify_user(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    path: web::Path<String>,
    auth_params: AuthParameters,
) -> HttpResponse {
   HttpResponse::Ok().finish()
}