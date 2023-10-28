use actix_web::{HttpMessage, HttpRequest, HttpResponse, web, get};
use actix_web::http::header::ContentType;
use anyhow::{anyhow, Error};
use sqlx::PgPool;
use uuid::Uuid;
use crate::authentication::AuthPermissions;
use crate::domain::app_user::DbUser;

#[derive(Clone)]
enum UserFilter {
    None,
    ByUsername(String),
    ByEmail(String),
    ByUuid(Uuid)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetUsersRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub user_id: Option<Uuid>
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct UserResponse {
    pub unique_id: Option<Uuid>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub role_id: Option<i32>,
    pub role_title: Option<String>
}

#[tracing::instrument(
name = "handle_get_users",
skip(pool, args),
)]
#[get("")]
pub async fn handle_get_users(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    args: web::Query<GetUsersRequest>,
) -> HttpResponse {
    let req_ext = req.extensions_mut();
    let auth_permissions: &AuthPermissions = req_ext.get::<AuthPermissions>().unwrap();
    get_user(pool, &args, &auth_permissions).await
}

pub async fn get_user(
    pool: web::Data<PgPool>,
    args: &GetUsersRequest,
    auth: &AuthPermissions
) -> HttpResponse {
    if args.email.is_none() && args.username.is_none() && args.user_id.is_none() {
        return HttpResponse::BadRequest().finish()
    }
    let mut filter_by = UserFilter::None;
    let mut extra_rights = false;
    if args.username.is_some() {
        filter_by = UserFilter::ByUsername(args.username.clone().unwrap());
        if auth.username == args.username.clone().unwrap() {
            extra_rights = true;
        }
    }
    else if args.user_id.is_some() {
        filter_by = UserFilter::ByUuid(args.user_id.clone().unwrap());
        if auth.username == args.username.clone().unwrap() {
            extra_rights = true;
        }
    }
    else if args.email.is_some() {
        filter_by = UserFilter::ByEmail(args.email.clone().unwrap())
    }

    let attempt = match filter_by.clone() {
        UserFilter::None => Err(anyhow!("Uh what?")),
        UserFilter::ByUuid(x) => get_db_user_with_id(&pool, Uuid::from(x.clone())).await,
        UserFilter::ByEmail(x) => get_db_user_with_email(&pool, x.clone()).await,
        UserFilter::ByUsername(x) => get_db_user_with_username(&pool, x.clone()).await,
    };
    let user_val = match attempt {
        Ok(x) => match x {
            Some(y) => y,
            None => return HttpResponse::Ok().finish()
        },
        Err(_) => return HttpResponse::InternalServerError().finish()
    };
    if extra_rights {
        // All the goods
        let user_resp = UserResponse {
            unique_id: Some(user_val.unique_id),
            username: Some(user_val.username),
            role_id: Some(user_val.role_id),
            email: Some(user_val.email),
            role_title: Some(user_val.role_title)
        };
        let json = serde_json::to_string(&user_resp).unwrap();
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(json);
    }
    let filter_again = filter_by.clone();
    // Restricted response
    let user_resp = match filter_again {
        UserFilter::None => return HttpResponse::InternalServerError().finish(),
        UserFilter::ByUsername(_) => {
            UserResponse { unique_id: None, email: None,
                username: Some(user_val.username), role_id: None, role_title: None,
            }
        }
        UserFilter::ByEmail(_) => {
            UserResponse {
                unique_id: None, email: Some(user_val.email),
                username: None, role_id: None, role_title: None,
            }
        }
        UserFilter::ByUuid(_) => {
            UserResponse { unique_id: None, email: None,
                username: Some(user_val.username), role_id: None, role_title: None,
            }
        }
    };
    let json = serde_json::to_string(&user_resp).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json)
}

// Hard-coded to the moon to avoid SQL injection
pub async fn get_db_user_with_id(
    pool: &PgPool,
    u_id: Uuid,
) -> Result<Option<DbUser>, anyhow::Error> {
    let user = sqlx::query_as!(
        DbUser,
       r#"SELECT usr.id AS unique_id,
        usr.email as email,
        usr.username as username,
        usr.phash as phash,
        usr.salt as salt,
        rls.id as role_id,
        rls.title as role_title
        FROM users usr
        INNER JOIN user_roles usr_rls ON usr.id = usr_rls.user_id
        INNER JOIN roles rls ON rls.id = usr_rls.role_id
        WHERE usr.id = $1; "#
        , u_id).fetch_optional(pool)
        .await
        .expect("Failed to perform a query to retrieve the user.");
    Ok(user)
}

// Hard-coded to the moon to avoid SQL injection
pub async fn get_db_user_with_username(
    pool: &PgPool,
    user_field: String,
) -> Result<Option<DbUser>, anyhow::Error> {
    let user = sqlx::query_as!(
        DbUser,
       r#"SELECT usr.id AS unique_id,
        usr.email as email,
        usr.username as username,
        usr.phash as phash,
        usr.salt as salt,
        rls.id as role_id,
        rls.title as role_title
        FROM users usr
        INNER JOIN user_roles usr_rls ON usr.id = usr_rls.user_id
        INNER JOIN roles rls ON rls.id = usr_rls.role_id
        WHERE usr.username = $1; "#
        , user_field).fetch_optional(pool)
        .await
        .expect("Failed to perform a query to retrieve the user.");
    Ok(user)
}

// Hard-coded to the moon to avoid SQL injection
pub async fn get_db_user_with_email(
    pool: &PgPool,
    user_field: String,
) -> Result<Option<DbUser>, anyhow::Error> {
    let user = sqlx::query_as!(
        DbUser,
       r#"SELECT usr.id AS unique_id,
        usr.email as email,
        usr.username as username,
        usr.phash as phash,
        usr.salt as salt,
        rls.id as role_id,
        rls.title as role_title
        FROM users usr
        INNER JOIN user_roles usr_rls ON usr.id = usr_rls.user_id
        INNER JOIN roles rls ON rls.id = usr_rls.role_id
        WHERE usr.email = $1; "#
        , user_field).fetch_optional(pool)
        .await
        .expect("Failed to perform a query to retrieve the user.");
    Ok(user)
}