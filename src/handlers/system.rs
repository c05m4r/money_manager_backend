// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::json;

use crate::AppState;

/// Health check
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 503, description = "Service is unhealthy"),
    ),
    security(()),
)]
pub async fn health(state: web::Data<AppState>) -> HttpResponse {
    let backend = state.db.get_database_backend();
    let db_check = state
        .db
        .execute(Statement::from_string(backend, "SELECT 1".to_string()))
        .await;

    match db_check {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "healthy",
            "database": "up",
            "timestamp": Utc::now(),
        })),
        Err(error) => HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "database": "down",
            "timestamp": Utc::now(),
            "error": error.to_string(),
        })),
    }
}
