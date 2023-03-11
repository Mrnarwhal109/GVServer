use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use crate::routes::{health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool)
    -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the 'wrap' method on 'App'
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
        // Register the connection as part of the application state
            .app_data(db_pool.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}