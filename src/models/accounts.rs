// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountResponse {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub currency_code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateAccountDto {
    pub user_uuid: Uuid,
    #[validate(length(min = 1, max = 150))]
    pub name: String,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
    #[validate(length(equal = 3))]
    pub currency_code: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateAccountDto {
    pub user_uuid: Option<Uuid>,
    #[validate(length(min = 1, max = 150))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
    #[validate(length(equal = 3))]
    pub currency_code: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AccountListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub user_uuid: Option<Uuid>,
    pub currency_code: Option<String>,
    pub search: Option<String>,
}
