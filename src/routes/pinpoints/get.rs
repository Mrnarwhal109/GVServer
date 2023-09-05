use chrono::Utc;
use uuid::Uuid;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::http::header::ContentType;
use sqlx::PgPool;
use crate::domain::Pinpoint;
use crate::domain::PinpointData;
use serde_json;
use crate::authentication::{AuthService, AuthParameters, AuthPermissions};

pub async fn handle_get_all_pinpoints(
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
    auth_params: AuthParameters
) -> HttpResponse {
    println!("Auth params to handler: {}", auth_params.0);
    let permissions: AuthPermissions;
    match auth.authorize_request(auth_params) {
        Ok(p) => permissions = p,
        Err(_) => return HttpResponse::Unauthorized().finish()
    };
    return HttpResponse::Ok().finish();
    let db_data = get_all_db_pinpoints(&pool).await.unwrap();
    let json = serde_json::to_string(&db_data).unwrap();

    HttpResponse::Ok().
        content_type(ContentType::json())
        .body(json)
}

pub async fn get_all_db_pinpoints(
    pool: &PgPool,
) -> Result<Vec<PinpointData>, sqlx::Error> {
    todo!();
/*    let rows = sqlx::query!(
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
    */
}
