use std::net::TcpListener;
use gv_server::startup::run;
use gv_server::configuration::get_configuration;
use gv_server::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPool;
use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    match dotenvy::dotenv()
    {
        Ok(_) => {

        },
        Err(e) => {
            println!("Error in main function: {}", e);
        }
    }

    let subscriber = get_subscriber(
        "gv_server".into(), "info".into(), std::io::stdout
    );
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(
        &configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our server
    // We have removed the hard-code '8000' - it's now coming from our settings!
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listener = TcpListener::bind(address)?;
    // We retrieve the port assigned to us by the OS
    // let port = listener.local_addr().unwrap().port();

    run(listener, connection_pool)?.await
}
