use actix_web::{HttpResponse, web};
use actix_web::http::header::ContentType;
use anyhow::{anyhow, Error};
use sqlx::PgPool;
use crate::authentication::{AuthService, AuthParameters, AuthPermissions};
use crate::database_models::DbPinpoint;
use crate::domain::{Pinpoint};
use crate::domain::pinpoint::{GetPinpointRequest, GetPinpointResponse};

pub async fn handle_get_pinpoints(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    args: web::Json<GetPinpointRequest>,
    auth_params: AuthParameters,
) -> HttpResponse {
    println!("Auth params to handler: {}", auth_params.jwt);
    let permissions: AuthPermissions;
    match auth.validate_request(&auth_params) {
        Ok(p) => permissions = p,
        Err(_) => return HttpResponse::Unauthorized().finish()
    };
    if args.username.is_none() || args.username == Some(String::from("")) {
        println!("Branching to get_all_pinpoints(...)");
        return get_all_pinpoints(pool, auth, args, auth_params, permissions).await;
    }
    else if args.latitude.is_some() && args.longitude.is_some() && args.radius.is_some() {
        // Later, replace with a fresh "get_all_proximity_pinpoints(...)" function.
        println!("Branching to get_all_proximity_pinpoints(...)");
        return get_all_user_pinpoints(pool, auth, args, auth_params, permissions).await;
    }
    else if args.username.is_some() {
        println!("Branching to get_all_user_pinpoints(...)");
        return get_all_user_pinpoints(pool, auth, args, auth_params, permissions).await;
    }
    HttpResponse::InternalServerError().finish()
}

pub async fn get_all_pinpoints(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    args: web::Json<GetPinpointRequest>,
    auth_params: AuthParameters,
    permissions: AuthPermissions,
) -> HttpResponse {
    let pinpoints = get_all_db_pinpoints(&pool).await.unwrap();
    let mut response_pinpoints: Vec<GetPinpointResponse> = Vec::new();
    for pinpoint in pinpoints.iter()
    {
        let convert = match GetPinpointResponse::try_from(pinpoint) {
            Ok(x) => x,
            Err(_) => return HttpResponse::InternalServerError().finish()
        };
        response_pinpoints.push(convert);
    }
    let filtered_pinpoints: Vec<GetPinpointResponse>
        = censor_pinpoints_by_username(&response_pinpoints, permissions.username.as_str());

    let vec_len = filtered_pinpoints.len();
    println!("Sending {} pinpoints back from handler", vec_len);
    let json = serde_json::to_string(&filtered_pinpoints).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json)
}

pub async fn get_all_db_pinpoints(
    pool: &PgPool,
) -> Result<Vec<Pinpoint>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DbPinpoint,
        r#"SELECT pin.id AS pinpoint_id, pin.latitude AS latitude, pin.longitude as longitude,
        pin.added_at AS added_at,
        con.id AS contents_id,
        con.description AS description,
        con.attachment AS attachment,
        usr.id AS user_id,
        usr.username AS username
        FROM pinpoints pin
        INNER JOIN pinpoint_contents pin_con on pin_con.pinpoint_id = pin.id
        INNER JOIN contents con ON con.id = pin_con.content_id
        INNER JOIN user_pinpoints usr_pin ON usr_pin.pinpoint_id = pin.id
        INNER JOIN users usr ON usr_pin.user_id = usr.id;
        "#).fetch_all(pool)
        .await
        .expect("Failed to perform a query to retrieve pinpoints.");

    let mut results: Vec<Pinpoint> = Vec::new();
    for row in rows.iter() {
        let pinpoint = Pinpoint::try_from(row)
            .map_err(|_| anyhow!("Conversion failure"))?;
        results.push(pinpoint);
    }

     for r in results.as_slice() {
         let lat = r.latitude.clone();
         let log = r.longitude.clone();
         let desc = r.description.clone();

         println!("Pinpoints converted from DB: latitude {}, longitude {}, description {}", lat, log, desc);
     }

    Ok(results)
}

pub async fn get_all_user_pinpoints(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    args: web::Json<GetPinpointRequest>,
    auth_params: AuthParameters,
    permissions: AuthPermissions,
) -> HttpResponse {
    let found_username = args.0.username.unwrap();
    let pinpoints = match get_all_user_db_pinpoints(
        &pool, found_username.as_str()).await {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    let response_pinpoints =
    match convert_to_pinpoint_response(pinpoints) {
        Ok(x) => x,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    let filtered_pinpoints: Vec<GetPinpointResponse>
        = censor_pinpoints_by_username(&response_pinpoints, permissions.username.as_str());

    let vec_len = filtered_pinpoints.len();
    println!("Sending {} pinpoints back from handler", vec_len);
    let json = serde_json::to_string(&filtered_pinpoints).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json)
}

pub async fn get_all_user_db_pinpoints(
    pool: &PgPool,
    username: &str
) -> Result<Vec<Pinpoint>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DbPinpoint,
        r#"SELECT pin.id AS pinpoint_id, pin.latitude AS latitude, pin.longitude as longitude,
        pin.added_at AS added_at,
        con.id AS contents_id,
        con.description AS description,
        con.attachment AS attachment,
        usr.id AS user_id,
        usr.username AS username
        FROM pinpoints pin
        INNER JOIN pinpoint_contents pin_con on pin_con.pinpoint_id = pin.id
        INNER JOIN contents con ON con.id = pin_con.content_id
        INNER JOIN user_pinpoints usr_pin ON usr_pin.pinpoint_id = pin.id
        INNER JOIN users usr ON usr_pin.user_id = usr.id
        WHERE usr.username = $1;
        "#, username).fetch_all(pool)
        .await
        .expect("Failed to perform a query to retrieve pinpoints.");
    let mut results: Vec<Pinpoint> = Vec::new();
    for row in rows.iter() {
        let pinpoint = Pinpoint::try_from(row)
            .map_err(|_| anyhow!("Conversion failure"))?;
        results.push(pinpoint);
    }
    for r in results.as_slice() {
        let lat = r.latitude.clone();
        let log = r.longitude.clone();
        let desc = r.description.clone();

        println!("Pinpoints converted from DB: latitude {}, longitude {}, description {}", lat, log, desc);
    }
    Ok(results)
}

pub fn convert_to_pinpoint_response(
    pinpoints: Vec<Pinpoint>
) -> Result<Vec<GetPinpointResponse>, anyhow::Error> {
    let mut response_pinpoints: Vec<GetPinpointResponse> = Vec::new();
    for pinpoint in pinpoints.iter() {
        let convert = match GetPinpointResponse::try_from(pinpoint) {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("Pinpoint conversion error"))
        };
        response_pinpoints.push(convert);
    }
    Ok(response_pinpoints)
}

pub fn censor_pinpoints_by_username(
    response_pinpoints: &Vec<GetPinpointResponse>,
    username: &str
) -> Vec<GetPinpointResponse> {
    let mut filtered_pinpoints: Vec<GetPinpointResponse> = Vec::new();
    // Remove more sensitive data about the pinpoint objects
    // when they aren't the requesting user's pinpoints
    for response_pinpoint in response_pinpoints {
        if response_pinpoint.pinpoint_username.is_some()
            && response_pinpoint.pinpoint_username.clone().unwrap() != username {
            filtered_pinpoints.push(response_pinpoint.clone_as_censored());
        }
        else {
            filtered_pinpoints.push(response_pinpoint.clone());
        }
    }
    filtered_pinpoints
}