use crate::authentication::{AuthService, basic_authentication};
use crate::authentication::{validate_credentials};
use actix_web::{HttpRequest, web};
use actix_web::HttpResponse;
use reqwest::StatusCode;
use sqlx::PgPool;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginData {
    pub username: String,
    pub pw: String,
}

#[tracing::instrument(
skip(pool, auth),
fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
// We are now injecting `PgPool` to retrieve stored credentials from the database
pub async fn handle_login(
    request: HttpRequest,
    pool: web::Data<PgPool>,
    auth: web::Data<AuthService>,
) -> Result<HttpResponse, actix_web::Error> {
    let credentials = match basic_authentication(&request.headers()) {
        Ok(c) => c,
        Err(_) => return Ok(HttpResponse::BadRequest().finish())
    };

    tracing::Span::current().record(
        "username", &tracing::field::display(credentials.clone().username.to_string()));

    match validate_credentials(credentials.clone(), &pool).await {
        Ok(_) => {
            let auth_jwt = auth.create_jwt(
                credentials.clone().username.to_string().as_str()).await;
            let auth_json = serde_json::json!({
                "jwt": auth_jwt
                });
            println!("Sending JSON auth response body: {}", auth_json.to_string());
            let good_response = HttpResponse::build(StatusCode::OK)
                .json(auth_json);
            Ok(good_response)
        },
        Err(_) => Ok(HttpResponse::BadRequest().finish())
    }
}