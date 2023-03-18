pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;
pub mod domain;

use actix_web::{web, App, HttpResponse, HttpServer};
