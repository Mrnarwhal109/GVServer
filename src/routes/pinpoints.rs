use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use crate::domain::NewPinpoint;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PinpointData {
    latitude: f64,
    longitude: f64,
    description: String
}

// Conversion spelled out for PinpointData into NewPinpoint
impl TryFrom<PinpointData> for NewPinpoint {
    type Error = String;
    fn try_from(value: PinpointData) -> Result<Self, Self::Error> {
        let latitude = value.latitude;
        let longitude = value.longitude;
        let description = value.description;
        Ok(Self { latitude, longitude, description })
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
    pool: &PgPool,
) -> Result<String, actix_web::Error>  {
    Ok(String::new())
}

#[tracing::instrument(
name = "Saving new pinpoint details in the database",
skip(new_pinpoint, pool)
)]
pub async fn insert_pinpoint(
    pool: &PgPool,
    new_pinpoint: &NewPinpoint,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO pinpoints (id, latitude, longitude, description, added_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        Uuid::new_v4(),
        new_pinpoint.latitude,
        new_pinpoint.longitude,
        new_pinpoint.description,
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

/*
pub async fn get_all_db_pinpoints(
    pool: &PgPool,
) -> Result<Vec<PinpointData>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, coordinates, description, added_at FROM pinpoints
        "#
    )
        .fetch_one(pool)
        .await
        .context("Failed to perform a query to retrieve pinpoints.")?;

    let mut results: Vec<PinpointData> = Vec::new();
    print!("MANUAL USER PRINT: ");
    println!("{}", rows);
    //for r in rows.iter() {
    //    let innerItem = r;
    //    results.append(innerItem);
    //}
    Ok(results)
}
*/
