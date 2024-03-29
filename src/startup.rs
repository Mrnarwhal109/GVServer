use crate::configuration::{DatabaseSettings, Settings};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use secrecy::{Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use actix_web_lab::middleware::from_fn;
use tracing_actix_web::TracingLogger;
use crate::authentication::AuthService;
use crate::authentication::middleware::get_jwt_permissions;
use crate::routes::health_check;
use crate::routes::login::handle_login;
use crate::routes::pinpoints::{handle_add_pinpoint, handle_get_pinpoints};
use crate::routes::pinpoints::delete::delete_routing::handle_delete_pinpoints;
use crate::routes::users::delete::delete_routing::handle_delete_user;
use crate::routes::users::get::handle_get_users;
use crate::routes::users::post::post_routing::{handle_signup};
use crate::routes::users::put::handle_put_user;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let auth_service = get_auth_service(&configuration);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        println!("Server running at address {}", address.clone());
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
            auth_service,
        )
            .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn get_auth_service(configuration: &Settings) -> AuthService {
    AuthService::new(configuration.application.jwt_secret.clone())
}

pub struct ApplicationBaseUrl(pub String);

async fn run(
    listener: TcpListener,
    db_pool: PgPool,
    base_url: String,
    auth_service: AuthService,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let auth_service = Data::new(auth_service);
    //let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    //let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let json_config = web::JsonConfig::default()
        .limit(20000000);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            //.wrap(from_fn(implant_token))
            .route("/", web::get().to(health_check))
            .route("/health_check", web::get().to(health_check))
            .route("/login", web::post().to(handle_login))
            .route("/users", web::get().to(handle_get_users))
            .route("/users", web::post().to(handle_signup))
            .service(
                web::scope("/pinpoints")
                    .wrap(from_fn(get_jwt_permissions))
                    .route("", web::post().to(handle_add_pinpoint))
                    .route("", web::delete().to(handle_delete_pinpoints))
                    .service(handle_get_pinpoints)
            )
            .service(
                web::scope("/users")
                    .wrap(from_fn(get_jwt_permissions))
                    .route("", web::delete().to(handle_delete_user))
                    .service(handle_put_user)
            )
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(json_config.clone())
            //.app_data(Data::new(HmacSecret(hmac_secret.clone())))
            .app_data(auth_service.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

