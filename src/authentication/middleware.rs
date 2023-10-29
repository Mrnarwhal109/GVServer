use actix_web::body::{MessageBody};
use actix_web::{App, Error, dev::{ServiceRequest, ServiceResponse, Service as _}, web};
use actix_web::{FromRequest, HttpMessage, HttpResponse};
use actix_web::error::{InternalError};
use actix_web::web::Data;
use actix_web_lab::middleware::Next;
use crate::authentication::{AuthParameters, AuthService};

pub async fn get_jwt_permissions(
    auth_service: web::Data<AuthService>,
    auth_params: AuthParameters,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    match auth_service.validate_request(&auth_params) {
        Ok(x) => {
            req.extensions_mut().insert(x);
            next.call(req).await
        },
        Err(_) => {
            let response = HttpResponse::Unauthorized().finish();
            let e = anyhow::anyhow!("Invalid authorization.");
            Err(InternalError::from_response(e, response).into())
        }
    }
}

