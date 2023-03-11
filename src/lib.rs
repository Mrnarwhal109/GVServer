pub mod configuration;
pub mod routes;
pub mod startup;
pub mod telemetry;

use actix_web::{web, App, HttpResponse, HttpServer};
