use actix_web::{HttpRequest};
use chrono::{DateTime, Duration, Utc};
use secrecy::{ExposeSecret, Secret};
use futures::future::{Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation, encode, Header, EncodingKey};
use crate::authentication::auth_permissions::AuthPermissions;
use crate::authentication::auth_token::Claims;
use crate::authentication::{AuthParameters, AuthPermissionsMode, validate_credentials};

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

    pub async fn create_jwt(&self, username: &str) -> String {
        let key = self.jwt_key.expose_secret().as_bytes();

        // Tokens expire in 7 days. This is subject to change.
        let mut _date: DateTime<Utc> = Utc::now() + Duration::days(7);

        let my_claims = Claims {
            sub: String::from(username),
            exp: _date.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &my_claims,
            &EncodingKey::from_secret(key),
        ).unwrap();
        token
    }

    pub fn validate_request(&self, auth_params: &AuthParameters) -> Result<AuthPermissions, String> {
        let key = self.jwt_key.expose_secret().as_bytes();
        let jwt_info = &auth_params.jwt.clone();
        println!("Attempting to authorize JWT {} using key {}", jwt_info, self.jwt_key.expose_secret());
        match decode::<Claims>(
            &auth_params.jwt,
            &DecodingKey::from_secret(key),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(_token) => {
                println!("JWT decoded, values are {} and {}.",
                         _token.claims.sub, _token.claims.exp);
                Ok(AuthPermissions::new(AuthPermissionsMode::None, _token.claims.sub))
            },
            Err(e) => {
                println!("JWT parsing failure; token not decoded properly! Error given: {}", e.to_string());
                Err(String::from("JWT parsing failure"))
            },
        }
    }

    pub fn validate_request_for_user(
        &self, auth_params: &AuthParameters, username: String
    ) -> Result<AuthPermissions, String> {
        let initial_validation = self.validate_request(auth_params)?;
        if initial_validation.username == username {
            println!("Validation of jwt against user succeeded.");
            return Ok(initial_validation);
        }
        println!("Validation of jwt against user failed.");
        Err(String::from("Unauthorized action for user."))
    }
}