use std::net::TcpListener;
use gv_server::startup::run;
use gv_server::configuration::get_configuration;
use sqlx::{Connection, PgConnection};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv();

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(
        &configuration.database.connection_string()
    )
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
