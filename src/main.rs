use gv_server::startup::build;
use gv_server::configuration::get_configuration;
use gv_server::telemetry::{get_subscriber, init_subscriber};

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

    let server = build(configuration).await?;
    server.await?;
    Ok(())
}
