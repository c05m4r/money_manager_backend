// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct AuditLogResponse {
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub action: String,
    pub table_name: String,
    pub description: Option<String>,
    pub record_uuid: Uuid,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
    pub ip_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct AuditLogListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub user_uuid: Option<Uuid>,
    pub action: Option<String>,
    pub table_name: Option<String>,
    pub record_uuid: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub ip_address: Option<String>,
}
