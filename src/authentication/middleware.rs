use crate::utils::{e500, see_other};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::{FromRequest, HttpMessage};
use actix_web_lab::middleware::Next;
use std::ops::Deref;
use uuid::Uuid;
use crate::authentication::AuthParameters;

pub async fn implant_token(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    println!("The middleware is running! implant_token()");
    let auth_token = {
        let (http_request, payload) = req.parts_mut();
        AuthParameters::from_request(http_request, payload).await
    }?;
    req.extensions_mut().insert(auth_token);
    next.call(req).await
}

