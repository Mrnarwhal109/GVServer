use std::io::Error;
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, FromRequest, HttpRequest};
use secrecy::{ExposeSecret, Secret};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use crate::authentication::auth_permissions::AuthPermissions;
use crate::authentication::auth_token::Claims;
use crate::authentication::{AuthParameters, AuthPermissionsMode};
use crate::domain::app_user::AppUser;

pub struct AuthService {
    jwt_key: Secret<String>
}

impl AuthService {
    pub fn new(
        jwt_key: Secret<String>
    ) -> Self {
        Self {
            jwt_key
        }
    }

    pub fn authorize_request(&self, auth_params: AuthParameters) -> Result<AuthPermissions, String> {
        let key = self.jwt_key.expose_secret().as_bytes();
        println!("Token key given: {}", self.jwt_key.expose_secret());
        match decode::<Claims>(
            &auth_params.0,
            &DecodingKey::from_secret(key),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(_token) => {
                println!("Token decoded, values are {} and {}.",
                         _token.claims.sub, _token.claims.exp);
                Ok(AuthPermissions::new(AuthPermissionsMode::None))
            },
            Err(e) => {
                println!("JWT parsing failure; token not decoded properly! Error given: {}", e.to_string());
                Err(String::from("JWT parsing failure"))
            },
        }
    }
}