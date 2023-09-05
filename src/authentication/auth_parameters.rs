use actix_session::SessionExt;
use actix_session::{Session, SessionGetError, SessionInsertError};
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use actix_web_lab::__reexports::futures_util::future::err;
use uuid::Uuid;

pub struct AuthParameters(pub(crate) String);

impl AuthParameters {
}

impl FromRequest for AuthParameters {
    type Error = Error;
    type Future = Ready<Result<AuthParameters, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        println!("from_request called!");
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[0].trim();
                println!("Token found: {}", token);
                ready(Ok(AuthParameters(token.to_string())))
            },
            None => ready(Ok(AuthParameters(String::from(""))))
        }
    }
}
