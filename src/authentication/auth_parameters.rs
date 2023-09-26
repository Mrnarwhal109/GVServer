use actix_web::{dev, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthParameters {
    pub jwt: String
}

impl AuthParameters {
}

impl FromRequest for AuthParameters {
    type Error = Error;
    type Future = Ready<Result<AuthParameters, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[0].trim();
                ready(Ok(AuthParameters { jwt: token.to_string() }))
            },
            None => ready(Ok(AuthParameters { jwt: String::from("") }))
        }
    }
}
