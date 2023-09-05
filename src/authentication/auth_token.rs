use serde::{Deserialize, Serialize};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}