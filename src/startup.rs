use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web::web::Data;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;
use crate::configuration::Settings;
use sqlx::postgres::PgPoolOptions;
use crate::configuration::DatabaseSettings;
use crate::routes::{confirm, health_check, publish_newsletter, subscribe, home, login_form, login};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    // Wrap the connection in a smart pointer
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the 'wrap' method on 'App'
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
        // Register the connection as part of the application state
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}


pub fn get_connection_pool(
    configuration: &DatabaseSettings
) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

// We need to define a wrapper type in order to retrieve the URL
// in the 'subscribe' handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw 'String' would expose us to conflicts.
pub struct ApplicationBaseUrl(pub String);

impl ToString for ApplicationBaseUrl {
    fn to_string(&self) -> String {
        return self.0.to_string();
    }
}

// A new type to hold the newly built server and its port
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    // We have converted the 'build' function into a constructor for
    // 'Application'.

    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        // No longer async, given that we don't actually try to connect
        let connection_pool = get_connection_pool(&configuration.database);

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
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
        )?;

        // We "save" the bound port in one of 'Application''s fields
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    // A more expressive name that makes it clear that
    // this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}