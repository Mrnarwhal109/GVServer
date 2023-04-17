use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpResponse, web};
use actix_web::http::header::ContentType;
use sqlx::PgPool;
use crate::domain::Pinpoint;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PinpointData {
    latitude: f64,
    longitude: f64,
    description: String,
    username: String
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TempUsername {
    username: String,
}

impl TryFrom<TempUsername> for String {
    type Error = String;
    fn try_from(value: TempUsername) -> Result<Self, Self::Error> {
        Ok(value.username.to_string())
    }
}

impl PinpointData {
    pub fn new(
        latitude: f64,
        longitude: f64,
        description: String,
        username: String,
    ) -> Self {
        Self {
            latitude,
            longitude,
            description,
            username
        }
    }
}

// Conversion spelled out for PinpointData into NewPinpoint
impl TryFrom<PinpointData> for Pinpoint {
    type Error = String;
    fn try_from(value: PinpointData) -> Result<Self, Self::Error> {
        let latitude = value.latitude;
        let longitude = value.longitude;
        let description = value.description;
        let username = value.username;
        Ok(Self { latitude, longitude, description, username })
    }
}

#[tracing::instrument(
name = "Adding a new pinpoint",
skip(pinpoint, pool),
fields(
pinpoint_latitude = %pinpoint.latitude,
pinpoint_longitude = %pinpoint.longitude
)
)]
pub async fn add_pinpoint(
    pinpoint: web::Json<PinpointData>,
    // Retrieving a connection from the application state!
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // 'web::Form' is a wrapper around 'FormData'
    // 'form.0' gives us access to the underlying 'FormData'
    // You can use NewSubscriber::try_from(form.0);
    let new_pinpoint = match pinpoint.0.try_into() {
        Ok(pinpoint) => pinpoint,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_pinpoint(&pool, &new_pinpoint).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

pub async fn get_all_pinpoints(
    pool: web::Data<PgPool>,
) -> HttpResponse {
    let db_data = get_all_db_pinpoints(&pool).await.unwrap();
    let json = serde_json::to_string(&db_data).unwrap();

    HttpResponse::Ok().
        content_type(ContentType::json())
        .body(json)
}

#[tracing::instrument(
name = "HTTP route : Delete all pinpoints",
skip(pool)
)]
pub async fn delete_all_pinpoints(
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match delete_all_db_pinpoints(&pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "HTTP route : Delete all of a user's pinpoints",
skip(pool, body)
)]
pub async fn delete_all_user_pinpoints(
    pool: web::Data<PgPool>,
    body: web::Json<TempUsername>,
) -> HttpResponse {

    let username: String = match body.0.try_into() {
        Ok(body) => body,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match delete_all_user_db_pinpoints(&pool, &username).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

#[tracing::instrument(
name = "Saving new pinpoint details in the database",
skip(new_pinpoint, pool)
)]
pub async fn insert_pinpoint(
    pool: &PgPool,
    new_pinpoint: &Pinpoint,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO pinpoints (id, latitude, longitude, description, username, added_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        Uuid::new_v4(),
        new_pinpoint.latitude,
        new_pinpoint.longitude,
        new_pinpoint.description,
        new_pinpoint.username,
        Utc::now()
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

pub async fn get_all_db_pinpoints(
    pool: &PgPool,
) -> Result<Vec<PinpointData>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, latitude, longitude, description, username, added_at FROM pinpoints
        "#
    )
        .fetch_all(pool)
        .await
        .expect("Failed to perform a query to retrieve pinpoints.");

    let mut results: Vec<PinpointData> = Vec::new();
    for r in rows.iter() {
        let p = PinpointData::new(
            r.latitude.unwrap(),
            r.longitude.unwrap(),
            r.description.clone().unwrap().to_string(),
            r.username.clone().unwrap().to_string()
        );
        results.push(p);
    }

    // for r in results.as_slice() {
    //     let lat = r.latitude.clone();
    //     let log = r.longitude.clone();
    //     let desc = r.description.clone();
    //
    //     println!("MANUAL PRINT latitude {}, longitude {}, description {}", lat, log, desc);
    // }

    Ok(results)
}

#[tracing::instrument(
name = "Deleting all pinpoints in the database",
skip(pool)
)]
pub async fn delete_all_db_pinpoints(
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM pinpoints;
        "#
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

#[tracing::instrument(
name = "Deleting all pinpoints in the database",
skip(pool)
)]
pub async fn delete_all_user_db_pinpoints(
    pool: &PgPool,
    username: &String
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM pinpoints WHERE username = $1;
        "#,
        username
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}