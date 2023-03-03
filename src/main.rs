use std::net::TcpListener;
use gv_server::startup::run;
use gv_server::configuration::get_configuration;
use sqlx::{Connection, PgConnection};
use sqlx::PgPool;
use env_logger::Env;

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

    // 'init' does call 'set_logger', so this is all we need to do.
    // We are falling back to printing all logs at info-level or above
    // if the RUST_LOG environment variable has not been set.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
