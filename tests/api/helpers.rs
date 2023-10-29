use std::io::Cursor;
use anyhow::anyhow;
use image::{ColorType, DynamicImage, ImageFormat, ImageResult};
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use gvserver::authentication::{AuthParameters, AuthService};
use gvserver::configuration::{get_configuration, DatabaseSettings};
use gvserver::database_models::db_user::DbUser;
use gvserver::routes::login::post::LoginData;
use gvserver::startup::{get_connection_pool, Application};
use gvserver::telemetry::{get_subscriber, init_subscriber};
use image::io::Reader;
use gvserver::domain::user_sign_up::UserSignUp;
use gvserver::routes::pinpoints::get::GetPinpointRequest;
use gvserver::routes::pinpoints::post::PostPinpointRequest;
use gvserver::routes::users::get::GetUsersRequest;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub auth_service: AuthService
}

impl TestApp {
    pub async fn create_jwt(&self, username: &str) -> String {
        self.auth_service.create_jwt(username).await
    }

    pub async fn get_users(&self, jwt: String, query: GetUsersRequest)
                               -> reqwest::Response {
        self.api_client
            .get(&format!("{}/users", &self.address))
            .header("Authorization", jwt)
            .query(&query)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_pinpoints(&self, jwt: String, username: String, query: GetPinpointRequest)
        -> reqwest::Response {
        self.api_client
            .get(&format!("{}/pinpoints/{}", &self.address, username))
            .header("Authorization", jwt)
            .query(&query)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_pinpoints(&self, jwt: String, body: PostPinpointRequest) -> reqwest::Response
    {
        let json_body = json!(body).to_string();
        self.api_client
            .post(&format!("{}/pinpoints", &self.address))
            .header("Content-Type", "application/json")
            .header("Authorization", jwt)
            .body(json_body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login(&self, username: String, pw: String) -> reqwest::Response
    {
        let response = self.api_client
            .post(&format!("{}/login", &self.address))
            .basic_auth(username, Some(pw))
            .send()
            .await
            .expect("Failed to execute request.");
        response
    }

    pub async fn post_signup(&self, json_data: String, username: String, pw: String) -> reqwest::Response
    {
        let response = self.api_client
            .post(&format!("{}/users", &self.address))
            .basic_auth(username, Some(pw))
            .header("Content-Type", "application/json")
            .body(json_data)
            .send()
            .await
            .expect("Failed to execute request.");
        response
    }

    pub async fn select_one_user(&self, username: String) -> Result<DbUser, anyhow::Error> {
        let user_rows = sqlx::query_as!(
        DbUser,
        "SELECT U.id AS unique_id, U.email AS email, U.username AS username, \
        U.phash AS phash, U.salt AS salt, R.id AS role_id, R.title AS role_title \
        FROM users U \
        INNER JOIN user_roles UR on UR.user_id = U.id \
        INNER JOIN roles R on UR.role_id = R.id \
        WHERE U.username = $1; ", username).fetch_one(&self.db_pool).await?;
        Ok(user_rows)
    }

    pub async fn sign_up_test_user(&self, username: &str, pw: &str, email: &str)
    -> String {
        let sign_up_data = UserSignUp {
            email: String::from(email),
            username: String::from(username),
            pw: String::from(pw)
        };
        let json_data = serde_json::to_string(&sign_up_data)
            .expect("Failed to serialize struct.");
        let response = self.post_signup(
            json_data, sign_up_data.username, sign_up_data.pw).await;
        let code = response.status().as_u16();
        let json_return = response.json::<AuthParameters>().await
            .expect("Failed to get a JSON response back.");
        let jwt = json_return.jwt.clone();
        assert_eq!(code, 200);
        jwt
    }

    pub async fn login_test(&self, username: &str, pw: &str) -> String {
        let login_data = LoginData {
            username: String::from(username),
            pw: String::from(pw)
        };
        serde_json::to_string(&login_data)
            .expect("Failed to serialize struct.");
        let response = self.post_login(
            login_data.username, login_data.pw).await;
        let code = response.status().as_u16();
        let json_return = response.json::<AuthParameters>().await
            .expect("Failed to get a JSON response back.");
        let jwt = json_return.jwt.clone();
        assert_eq!(code, 200);
        jwt
    }

    pub fn get_test_input_dir_path(&self) -> String {
        String::from("./input_container")
    }

    pub fn get_test_output_dir_path(&self) -> String {
        String::from("./output_container")
    }

    pub async fn load_img_at(&self, path: &str) -> Option<DynamicImage> {
        let img_opened = match image::io::Reader::open(path) {
            Ok(x) => x,
            Err(_) => {
                println!("No image loaded A");
                return None
            }
        };
        let img = match img_opened.decode() {
            Ok(x) => x,
            Err(_) => {
                println!("No image loaded B");
                return None
            }
        };
        Some(img)
    }

    pub async fn load_img_bytes_at(&self, path: &str) -> Option<Vec<u8>> {
        let img = self.load_img_at(path).await;
        if img.is_none() {
            println!("No image loaded C");
            return None;
        }
        let img_wr = img.unwrap();
        let img_cln = img_wr.clone();
        println!("Image colortype found: {:?}", img_cln.color());
        Some(img_wr.into_bytes())
        //img.map(|x| x.into_bytes())
        // Equivalent
        /*
        match self.load_img_at(path).await {
            None => None,
            Some(x) => Some(x.into_bytes())
        }
         */
    }

    pub async fn save_img_at(&self, path: &str, img: DynamicImage) -> ImageResult<()> {
        img.save(path)
    }

    pub async fn save_img_bytes_at(&self, path: &str, bytes: &Vec<u8>, img_quality: u8)
        -> Result<bool, anyhow::Error> {
        let b_count = bytes.len();
        //let buffer = vec![0; 150];
        println!("Bytes to save count: {:?}", b_count);
        let attempt = image::save_buffer_with_format(
            path, &bytes, 1280, 720, ColorType::Rgb8, ImageFormat::Jpeg);
        match attempt {
            Ok(_) => Ok(true),
            Err(_) => {
                println!("Save error C");
                Err(anyhow!("Failed to save image bytes."))
            }
        }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for Postmark's API
    // let email_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as email API
        // c.email_client.base_url = email_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let auth_service = AuthService::new(configuration.application.jwt_secret);

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        test_user: TestUser::generate(),
        api_client: client,
        auth_service
    };

    //test_app.test_user.store_new_user(&test_app.db_pool).await;

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub struct TestUser {
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    /*
    pub async fn login(&self, app: &TestApp) {
        app.post_login(&serde_json::json!({
            "username": &self.username,
            "password": &self.password
        }))
            .await;
    }

     */
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
