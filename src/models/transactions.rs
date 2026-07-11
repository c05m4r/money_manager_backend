// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub uuid: Uuid,
    pub amount: String,
    pub description: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub type_uuid: Uuid,
    pub category_uuid: Option<Uuid>,
    pub account_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTransactionDto {
    pub amount: String,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub type_uuid: Uuid,
    pub category_uuid: Option<Uuid>,
    pub account_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateTransactionDto {
    pub amount: Option<String>,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
    pub transaction_date: Option<DateTime<Utc>>,
    pub type_uuid: Option<Uuid>,
    pub category_uuid: Option<Uuid>,
    pub account_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct TransactionListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub user_uuid: Option<Uuid>,
    pub account_uuid: Option<Uuid>,
    pub category_uuid: Option<Uuid>,
    pub type_uuid: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub amount_min: Option<String>,
    pub amount_max: Option<String>,
    pub search: Option<String>,
}
