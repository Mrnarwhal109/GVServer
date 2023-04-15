use gv_server::configuration::get_configuration;
use gv_server::telemetry::{get_subscriber, init_subscriber};
use gv_server::startup::Application;

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

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
