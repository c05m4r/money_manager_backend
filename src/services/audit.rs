// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::audit_logs;
use crate::errors::AppError;

pub async fn write_audit_log(
    db: &DatabaseConnection,
    user_uuid: Uuid,
    action: &str,
    table_name: &str,
    record_uuid: Uuid,
    old_values: Option<Value>,
    new_values: Option<Value>,
    ip_address: String,
) -> Result<(), AppError> {
    let model = audit_logs::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        user_uuid: Set(user_uuid),
        action: Set(action.to_string()),
        table_name: Set(table_name.to_string()),
        description: Set(None),
        record_uuid: Set(record_uuid),
        old_values: Set(old_values),
        new_values: Set(new_values),
        ip_address: Set(ip_address),
        created_at: Set(Utc::now()),
    };

    model.insert(db).await?;
    Ok(())
}
