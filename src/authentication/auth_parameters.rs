use actix_web::{dev, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthParameters {
    pub jwt: String
}

impl AuthParameters {
    pub fn attempt_from_request(req: &HttpRequest, _payload: &mut dev::Payload)
                            -> AuthParameters {
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let test = _auth.unwrap().to_str().unwrap();
                println!("Auth unwrapped: {}", test);
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[0].trim();
                AuthParameters { jwt: token.to_string() }
            },
            None => AuthParameters { jwt: String::from("") }
        }
    }
}

impl FromRequest for AuthParameters {
    type Error = Error;
    type Future = Ready<Result<AuthParameters, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        ready(Ok(AuthParameters::attempt_from_request(req, _payload)))
    }
}
