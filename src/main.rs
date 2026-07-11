// Copyright (C) 2026 Marcos Gabriel Miller
mod config;
mod db;
mod entities;
mod errors;
mod handlers;
mod models;
mod services;

#[path = "../specs/openapi.rs"]
mod openapi;

use actix_web::{middleware::Logger, web, App, HttpServer};
use config::AppConfig;
use db::{init_db, run_migrations};
use openapi::ApiDoc;
use sea_orm::DatabaseConnection;
use services::authorization::AuthorizationEnforcer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub enforcer: AuthorizationEnforcer,
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(handlers::system::health))
            .service(
                web::scope("/auth")
                    .route("/forgot-password", web::post().to(handlers::auth::forgot_password))
                    .route("/reset-password", web::post().to(handlers::auth::reset_password))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/verify-email", web::get().to(handlers::auth::verify_email))
                    .route(
                        "/resend-verification-email",
                        web::post().to(handlers::auth::resend_verification_email),
                    )
                    .route(
                        "/change-password",
                        web::post().to(handlers::auth::change_password),
                    )
                    .route("/logout", web::post().to(handlers::auth::logout)),
            )
            .service(
                web::scope("/users")
                    .route("", web::get().to(handlers::users::list))
                    .route("", web::post().to(handlers::users::create))
                    .route("/{uuid}", web::get().to(handlers::users::get_one))
                    .route("/{uuid}", web::patch().to(handlers::users::update))
                    .route("/{uuid}", web::delete().to(handlers::users::delete)),
            )
            .service(
                web::scope("/currencies")
                    .route("", web::get().to(handlers::currencies::list))
                    .route("", web::post().to(handlers::currencies::create))
                    .route("/{code}", web::get().to(handlers::currencies::get_one))
                    .route("/{code}", web::patch().to(handlers::currencies::update))
                    .route("/{code}", web::delete().to(handlers::currencies::delete)),
            )
            .service(
                web::scope("/accounts")
                    .route("", web::get().to(handlers::accounts::list))
                    .route("", web::post().to(handlers::accounts::create))
                    .route("/{uuid}", web::get().to(handlers::accounts::get_one))
                    .route("/{uuid}", web::patch().to(handlers::accounts::update))
                    .route("/{uuid}", web::delete().to(handlers::accounts::delete)),
            )
            .service(
                web::scope("/categories")
                    .route("", web::get().to(handlers::categories::list))
                    .route("", web::post().to(handlers::categories::create))
                    .route("/{uuid}", web::get().to(handlers::categories::get_one))
                    .route("/{uuid}", web::patch().to(handlers::categories::update))
                    .route("/{uuid}", web::delete().to(handlers::categories::delete)),
            )
            .service(
                web::scope("/transaction-types")
                    .route("", web::get().to(handlers::transaction_types::list))
                    .route("", web::post().to(handlers::transaction_types::create))
                    .route("/{uuid}", web::get().to(handlers::transaction_types::get_one))
                    .route("/{uuid}", web::patch().to(handlers::transaction_types::update))
                    .route("/{uuid}", web::delete().to(handlers::transaction_types::delete)),
            )
            .service(
                web::scope("/transactions")
                    .route("", web::get().to(handlers::transactions::list))
                    .route("", web::post().to(handlers::transactions::create))
                    .route("/{uuid}", web::get().to(handlers::transactions::get_one))
                    .route("/{uuid}", web::patch().to(handlers::transactions::update))
                    .route("/{uuid}", web::delete().to(handlers::transactions::delete)),
            )
            .service(
                web::scope("/audit-logs")
                    .route("", web::get().to(handlers::audit_logs::list))
                    .route("/{uuid}", web::get().to(handlers::audit_logs::get_one)),
            ),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::from_filename("env/.env")
        .or_else(|_| dotenvy::dotenv())
        .ok();
    env_logger::init();

    let config = AppConfig::from_env();
    let db = init_db(&config.database_url).await;
    run_migrations(&db)
        .await
        .expect("failed to initialize schema and seed data");

    let enforcer = services::authorization::build_enforcer()
        .await
        .expect("failed to initialize authorization enforcer");

    let state = web::Data::new(AppState {
        db,
        config: config.clone(),
        enforcer,
    });

    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .configure(configure_routes)
            .service(
                SwaggerUi::new("/swagger/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind((config.host.clone(), config.port))?
    .run()
    .await
}
