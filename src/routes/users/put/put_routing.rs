use actix_web::{HttpMessage, HttpRequest, HttpResponse, put, web};
use actix_web::http::StatusCode;
use anyhow::{anyhow};
use secrecy::{ExposeSecret, Secret};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::{Uuid};
use crate::authentication::{AuthPermissions, AuthService, compute_password_hash, rand_salt_string};
use crate::domain::database::DbUser;
use crate::{ok_or_return_with, some_or_return_with};
use crate::routes::users::get::{get_db_user_with_id, get_db_user_with_username};
use crate::routes::users::put::put_user_request::PutUserRequest;
use crate::utils::options_eq;

impl TryFrom<(DbUser, &PutUserRequest)> for DbUser {
    type Error = anyhow::Error;

    fn try_from(value: (DbUser, &PutUserRequest)) -> Result<Self, Self::Error> {
        let mut result = value.0.clone();
        result.username = match &value.1.username {
            Some(x) => x.as_str().to_string(),
            None => value.0.username
        };
        result.email = match &value.1.email {
            Some(x) => x.as_str().to_string(),
            None => value.0.email
        };
        match &value.1.password {
            Some(x) => {
                let salt = rand_salt_string();
                let phash = compute_password_hash(
                    &Secret::new(x.as_str().to_string()), &salt)
                    .map_err(|e| anyhow!(e.to_string()))?;
                result.salt = salt.to_string();
                result.phash = phash.expose_secret().to_string();
            },
            None => {}
        };
        result.contents_description = match &value.1.contents_description {
            Some(x) => Some(x.as_str().to_string()),
            None => value.0.contents_description
        };
        if value.1.contents_attachment.is_some() {
            // There is likely a solution to prevent this clone.
            result.contents_attachment = value.1.contents_attachment.clone();
        }
        Ok(result)
    }
}

#[tracing::instrument(
name = "handle_put_users",
skip(pool, path, auth)
)]
#[put("/{user_id}")]
pub async fn handle_put_user(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    auth: web::Data<AuthService>,
    args: web::Json<PutUserRequest>
) -> HttpResponse {
    let req_ext = req.extensions_mut();
    let auth_permissions: &AuthPermissions = req_ext.get::<AuthPermissions>().unwrap();
    let user_id = path.into_inner();
    println!("User ID in path : {}", user_id.clone());
    let user_requesting = match get_db_user_with_id(&pool, user_id).await {
        Ok(x) => match x {
            None => {
                println!("TEST ERROR A");
                return HttpResponse::BadRequest().body("URL path contains non-existent user.");
            }
            Some(y) => y
        }
        Err(_) => {
            println!("TEST ERROR B");
            return HttpResponse::BadRequest().body("URL path contains non-existent user.");
        }
    };
    if auth_permissions.username != user_requesting.username.as_str() {
        return HttpResponse::Unauthorized().finish();
    }
    if args.is_empty() {
        println!("TEST ERROR C");
        return HttpResponse::BadRequest().finish();
    }
    modify_user(&pool, &user_requesting.username, &args.0).await;
    match args.0.username {
        Some(x) => {
            // We now need a new JWT that corresponds with the new username
            let auth_jwt = auth.create_jwt(
                x.as_str()).await;
            let auth_json = serde_json::json!({"jwt": auth_jwt});
            let good_response = HttpResponse::build(StatusCode::OK)
                .json(auth_json);
            good_response
        },
        None => HttpResponse::Ok().finish()
    }

}

pub async fn modify_user(pool: &PgPool, existing_username: &str, args: &PutUserRequest)
-> HttpResponse {
    let get_stored_result = ok_or_return_with!(
        get_db_user_with_username(pool, existing_username).await,
        HttpResponse::BadRequest().finish()
    );
    let existing_user = some_or_return_with!(
        get_stored_result, HttpResponse::BadRequest().finish());
    println!("modify user args {:?}", args);
    println!("existing user {:?}", existing_user);
    // A blank contents item in the request will be used to erase
    // the corresponding contents field in the database.
    // A null contents item in the request will do nothing.
    let modify_contents = (
        args.contents_attachment.is_some()
        && !options_eq(&existing_user.contents_attachment, &args.contents_attachment))
        ||
        (args.contents_description.is_some()
            && !options_eq(&existing_user.contents_description, &args.contents_description));
    let mut tran = ok_or_return_with!(
        pool.begin().await, HttpResponse::InternalServerError().finish());
    let username_change_request = args.username.clone();
    let email_change_request = args.email.clone();
    if username_change_request.is_some() || email_change_request.is_some() {
        let user_exists_check = stored_user_exists(&mut tran,
                                                   username_change_request, email_change_request).await;
        match user_exists_check {
            Ok(x) => {
                if x {
                    return HttpResponse::BadRequest().body(
                        "The username or email given is already in use.")
                }
            }
            Err(_) => {
                return HttpResponse::InternalServerError().finish();
            }
        }
    }
    // Hash and salt are assigned within the conversion
    let user_changes: DbUser = match (existing_user, args).try_into() {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };
     match modify_stored_user(
        &mut tran, existing_username, user_changes, modify_contents).await {
        Ok(_) => {
            match tran.commit().await {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(_) => HttpResponse::InternalServerError().finish()
            }
        },
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

async fn stored_user_exists(tran: &mut Transaction<'_, Postgres>,
                            check_username: Option<String>, check_email: Option<String>)
    -> Result<bool, anyhow::Error> {
    // These checks are quicker for the database.
    if check_username.is_none() && check_email.is_none() {
        return Err(anyhow!("Invalid query parameters."));
    }
    {
        let reborrow = &mut (*tran);
        if check_username.is_some() {
            let query_result = sqlx::query!(
            r#"
            SELECT usr.id
            FROM users usr
            WHERE usr.username = $1;
            "#, check_username.unwrap())
                .fetch_optional(reborrow)
                .await?;
            if query_result.is_some() {
                return Ok(true);
            }
        }
    }
    if check_email.is_some() {
        let query_result = sqlx::query!(
            r#"
            SELECT usr.id
            FROM users usr
            WHERE usr.email = $1;
            "#, check_email.unwrap())
            .fetch_optional(tran)
            .await?;
        if query_result.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn modify_stored_user(
    tran: &mut Transaction<'_, Postgres>,
    existing_username: &str, user_changes: DbUser,
    update_contents: bool
) -> Result<bool, sqlx::Error> {
    println!("Modification occurring: {:?}", user_changes.clone());
    let query_results = sqlx::query!(
            r#"
            WITH usr_id(id) AS (
                SELECT DISTINCT id FROM users WHERE username = $1
            ),
            usr AS (
                UPDATE users
                SET username = $2, email = $3, phash = $4, salt = $5
                WHERE id IN (SELECT id FROM usr_id)
            )
            SELECT contents_id FROM user_contents uc
            WHERE uc.user_id in (SELECT id FROM usr_id);
            "#,
            existing_username,
            user_changes.username,
            user_changes.email,
            user_changes.phash,
            user_changes.salt,
        )
        .fetch_optional(&mut (*tran))
        .await?;
    if query_results.is_some() && update_contents {
        let cnt_id = query_results.unwrap().contents_id;

        sqlx::query!(
            r#"
                UPDATE contents SET description = $1, attachment = $2
                WHERE id = $3;
            "#,
            user_changes.contents_description,
            user_changes.contents_attachment,
            cnt_id
        ).execute(tran).await?;
    }
    Ok(true)
}