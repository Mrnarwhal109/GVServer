use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use crate::routes::{add_pinpoint, get_all_pinpoints, health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;
use crate::configuration::Settings;
use sqlx::postgres::PgPoolOptions;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the 'wrap' method on 'App'
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            .route("/pinpoints", web::post().to(add_pinpoint))
            .route("/pinpoints", web::get().to(get_all_pinpoints))
        // Register the connection as part of the application state
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}

pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
    // No longer async, given that we don't actually try to connect
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    // Build an EmailClient using configuration
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        // Pass argument from configuration
        configuration.email_client.authorization_token,
        timeout,
    );

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our server
    // We have removed the hard-code '8000' - it's now coming from our settings!
    let address = format!(
        "{}:{}", configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(address)?;
    // We retrieve the port assigned to us by the OS
    // let port = listener.local_addr().unwrap().port();

    run(listener, connection_pool, email_client)
}