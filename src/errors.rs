// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    InternalServerError(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (error, message) = match self {
            AppError::NotFound(message) => ("NOT_FOUND", message.clone()),
            AppError::Unauthorized => ("UNAUTHORIZED", "Invalid credentials".to_string()),
            AppError::Forbidden => ("FORBIDDEN", "Insufficient permissions".to_string()),
            AppError::BadRequest(message) => ("BAD_REQUEST", message.clone()),
            AppError::Conflict(message) => ("CONFLICT", message.clone()),
            AppError::InternalServerError(message) => ("INTERNAL_ERROR", message.clone()),
        };

        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: error.to_string(),
            message,
        })
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(value: sea_orm::DbErr) -> Self {
        AppError::InternalServerError(value.to_string())
    }
}
