use sqlx::{PgPool, PgConnection, Connection, Executor};
use sqlx::types::uuid;
use gv_server::configuration::{ get_configuration, DatabaseSettings };
use gv_server::startup::run;
use gv_server::telemetry::{get_subscriber, init_subscriber};
use std::net::TcpListener;
use uuid::Uuid;
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
// 'tokio::test' is the testing equivalent of 'tokio::main'.
// It also spares you from having to specify the '#[test] attribute.

// Ensure that the 'tracing' stack is only initialised once using 'once_cell'
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of 'get_subscriber' to a variable based on the
    // value TEST_LOG' because the sink is part of the type returned by
    // 'get_subscriber', therefore they are not the same type. We could work around
    // it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout
        );
        init_subscriber(subscriber);
    }
    else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink
        );
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {

    // The first time 'initialize' is invoked, the code in 'TRACING' is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration()
        .expect("Failed to read configuration.");

    configuration.database.database_name = Uuid::new_v4().to_string();
    // println!("Manual print, database name is {}", configuration.database.database_name);

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone())
        .expect("Failed to bind address");

    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // println!("conn string without db: {}", &config.connection_string_without_db());
    // Create database
    let mut connection = PgConnection::connect_with(
        &config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(
        config.with_db()
    )
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

// You can inspect what macro code gets generated using
// 'cargo expand --test health_check' (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app()
        .await;
    // We need to bring in 'reqwest'
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    // We need to bring in 'reqwest'
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // query! macro returns an annonymous record type, compile-time generated struct
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customized error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}