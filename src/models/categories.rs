// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateCategoryDto {
    pub user_uuid: Uuid,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateCategoryDto {
    #[validate(length(min = 1, max = 120))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct CategoryListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub user_uuid: Option<Uuid>,
    pub search: Option<String>,
}
