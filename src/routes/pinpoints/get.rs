use actix_web::{get, HttpMessage, HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use anyhow::{anyhow};
use sqlx::{PgPool};
use crate::authentication::{AuthService, AuthParameters, AuthPermissions};
use crate::database_models::DbPinpoint;
use crate::domain::{Pinpoint};
use crate::domain::pinpoint::{GetPinpointRequest, GetPinpointResponse};


#[tracing::instrument(
name = "handle_get_pinpoints",
skip(pool, path, args),
)]
#[get("/{username}")]
pub async fn handle_get_pinpoints(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    args: web::Query<GetPinpointRequest>,
) -> HttpResponse {
    let req_ext = req.extensions_mut();
    let auth_permissions: &AuthPermissions = req_ext.get::<AuthPermissions>().unwrap();
    println!("AuthPermissions found as {:?}", auth_permissions);
    let user_requesting = path.into_inner();
    println!("GetPinpointRequest to handler: {}", args.0);
    if auth_permissions.username != user_requesting {
        return HttpResponse::Unauthorized().finish();
    }
    get_pinpoints(pool, Some(user_requesting), args).await
}

pub async fn get_pinpoints(
    pool: web::Data<PgPool>,
    user_requesting: Option<String>,
    args: web::Query<GetPinpointRequest>,
) -> HttpResponse {
    let user_filter: Option<String> = args.0.username;
    let latitude = args.0.latitude.unwrap_or(0.0);
    let longitude = args.0.longitude.unwrap_or(0.0);
    let proximity = args.0.proximity.unwrap_or(9999.0);

    let pinpoints = match get_db_pinpoints(
        &pool, user_filter, latitude, longitude, proximity).await {
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
        = censor_pinpoints_by_username(
        &response_pinpoints,
        user_requesting.unwrap_or(String::from("")).as_str());

    let vec_len = filtered_pinpoints.len();
    println!("Sending {} pinpoints back from handler", vec_len);
    let json = serde_json::to_string(&filtered_pinpoints).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json)
}


// Filters by username after the query
// if a username is present
pub async fn get_db_pinpoints(
    pool: &PgPool,
    user_filter: Option<String>,
    latitude: f64,
    longitude: f64,
    proximity: f64,
) -> Result<Vec<Pinpoint>, anyhow::Error> {
    let lat_lower = latitude.clone() - proximity.clone();
    let lat_upper = latitude.clone() + proximity.clone();
    let long_lower = longitude.clone() - proximity.clone();
    let long_upper = longitude + proximity;

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
        WHERE pin.latitude > $1 AND pin.latitude < $2
        AND pin.longitude > $3 AND pin.longitude < $4 "#
        , lat_lower, lat_upper, long_lower, long_upper).fetch_all(pool)
        .await
        .expect("Failed to perform a query to retrieve pinpoints.");
    let mut results: Vec<Pinpoint> = Vec::new();

    if user_filter.is_some() {
        let user_inner = user_filter.unwrap();
        for row in rows.iter() {
            let pinpoint = Pinpoint::try_from(row)
                .map_err(|_| anyhow!("Conversion failure"))?;
            if pinpoint.username == user_inner {
                results.push(pinpoint);
            }
        }
    }
    else {
        for row in rows.iter() {
            let pinpoint = Pinpoint::try_from(row)
                .map_err(|_| anyhow!("Conversion failure"))?;
            results.push(pinpoint);
        }
    }

    for r in results.as_slice() {
        let lat = r.latitude.clone();
        let log = r.longitude.clone();
        let desc = r.description.clone();
        let usr = r.username.clone();

        println!("Pinpoint converted from DB: latitude {}, longitude {}, \
        description {}, username {}", lat, log, desc, usr);
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
        if response_pinpoint.pinpoint_username.is_some() {
            if response_pinpoint.pinpoint_username.clone().unwrap() != username {
                filtered_pinpoints.push(response_pinpoint.clone_as_censored());
            }
            else {
                filtered_pinpoints.push(response_pinpoint.clone());
            }
        }
    }
    filtered_pinpoints
}