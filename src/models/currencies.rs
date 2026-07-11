// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrencyResponse {
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCurrencyDto {
    #[validate(length(equal = 3))]
    pub code: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 10))]
    pub symbol: String,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCurrencyDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 10))]
    pub symbol: Option<String>,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct CurrencyListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
}
