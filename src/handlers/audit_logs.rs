// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveTime, Utc};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};

use crate::{
    entities::audit_logs,
    errors::AppError,
    models::{
        audit_logs::{AuditLogListQuery, AuditLogResponse},
        pagination::PaginatedResponse,
    },
    AppState,
};

use super::utils::{authorize_action, paginated, AuthorizedUser};

fn end_of_day() -> NaiveTime {
    NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999).expect("valid end of day")
}

fn to_response(model: audit_logs::Model) -> AuditLogResponse {
    AuditLogResponse {
        uuid: model.uuid,
        user_uuid: model.user_uuid,
        action: model.action,
        table_name: model.table_name,
        description: model.description,
        record_uuid: model.record_uuid,
        old_values: model.old_values,
        new_values: model.new_values,
        ip_address: model.ip_address,
        created_at: model.created_at,
    }
}

/// List audit logs (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/audit-logs",
    tag = "Audit Logs",
    params(AuditLogListQuery),
    responses(
        (status = 200, description = "Paginated list of audit logs", body = inline(PaginatedResponse<AuditLogResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<AuditLogListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "audit_logs", "list", None).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = audit_logs::Entity::find();
    if let Some(user_uuid) = query.user_uuid {
        select = select.filter(audit_logs::Column::UserUuid.eq(user_uuid));
    }
    if let Some(action) = &query.action {
        select = select.filter(audit_logs::Column::Action.eq(action.to_uppercase()));
    }
    if let Some(table_name) = &query.table_name {
        select = select.filter(audit_logs::Column::TableName.eq(table_name));
    }
    if let Some(record_uuid) = query.record_uuid {
        select = select.filter(audit_logs::Column::RecordUuid.eq(record_uuid));
    }
    if let Some(ip_address) = &query.ip_address {
        select = select.filter(audit_logs::Column::IpAddress.contains(ip_address));
    }
    if let Some(date_from) = query.date_from {
        let from = DateTime::<Utc>::from_naive_utc_and_offset(date_from.and_time(NaiveTime::MIN), Utc);
        select = select.filter(audit_logs::Column::CreatedAt.gte(from));
    }
    if let Some(date_to) = query.date_to {
        let to = DateTime::<Utc>::from_naive_utc_and_offset(date_to.and_time(end_of_day()), Utc);
        select = select.filter(audit_logs::Column::CreatedAt.lte(to));
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<AuditLogResponse> =
        paginated(data, total, page, per_page, "/api/v1/audit-logs");
    Ok(HttpResponse::Ok().json(response))
}

/// Get an audit log entry by UUID
#[utoipa::path(
    get,
    path = "/api/v1/audit-logs/{uuid}",
    tag = "Audit Logs",
    params(("uuid" = uuid::Uuid, Path, description = "Audit log UUID")),
    responses(
        (status = 200, description = "Audit log found", body = AuditLogResponse),
        (status = 404, description = "Audit log not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "audit_logs", "get", None).await?;

    let row = audit_logs::Entity::find_by_id(path.into_inner())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Audit log not found".to_string()))?;
    Ok(HttpResponse::Ok().json(to_response(row)))
}
