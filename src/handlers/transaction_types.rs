// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entities::transaction_types,
    errors::AppError,
    models::{
        pagination::PaginatedResponse,
        transaction_types::{
            CreateTransactionTypeDto, TransactionTypeListQuery, TransactionTypeResponse,
            UpdateTransactionTypeDto,
        },
    },
    services::audit::write_audit_log,
    AppState,
};

use super::utils::{authorize_action, paginated, request_ip, validate_dto, AuthorizedUser};

fn to_response(model: transaction_types::Model) -> TransactionTypeResponse {
    TransactionTypeResponse {
        uuid: model.uuid,
        name: model.name,
        code: model.code,
        description: model.description,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List transaction types (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/transaction-types",
    tag = "Transaction Types",
    params(TransactionTypeListQuery),
    responses(
        (status = 200, description = "Paginated list of transaction types", body = inline(PaginatedResponse<TransactionTypeResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<TransactionTypeListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "transaction_types", "list", None).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select =
        transaction_types::Entity::find().filter(transaction_types::Column::DeletedAt.is_null());
    if let Some(search) = &query.search {
        select = select.filter(
            Condition::any()
                .add(transaction_types::Column::Name.contains(search))
                .add(transaction_types::Column::Code.contains(search)),
        );
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<TransactionTypeResponse> = paginated(
        data,
        total,
        page,
        per_page,
        "/api/v1/transaction-types",
    );
    Ok(HttpResponse::Ok().json(response))
}

/// Get a transaction type by UUID
#[utoipa::path(
    get,
    path = "/api/v1/transaction-types/{uuid}",
    tag = "Transaction Types",
    params(("uuid" = Uuid, Path, description = "Transaction type UUID")),
    responses(
        (status = 200, description = "Transaction type found", body = TransactionTypeResponse),
        (status = 404, description = "Transaction type not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "transaction_types", "get", None).await?;

    let row = transaction_types::Entity::find_by_id(path.into_inner())
        .filter(transaction_types::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction type not found".to_string()))?;
    Ok(HttpResponse::Ok().json(to_response(row)))
}

/// Create a new transaction type
#[utoipa::path(
    post,
    path = "/api/v1/transaction-types",
    tag = "Transaction Types",
    request_body = CreateTransactionTypeDto,
    responses(
        (status = 201, description = "Transaction type created", body = TransactionTypeResponse),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<CreateTransactionTypeDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "transaction_types", "create", None).await?;
    let actor_uuid = auth.uuid;

    let now = Utc::now();
    let row = transaction_types::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        name: Set(body.name.clone()),
        code: Set(body.code.to_uppercase()),
        description: Set(body.description.clone()),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "INSERT",
        "transaction_types",
        row.uuid,
        None,
        Some(json!({"uuid": row.uuid, "code": row.code})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Created().json(to_response(row)))
}

/// Update a transaction type
#[utoipa::path(
    patch,
    path = "/api/v1/transaction-types/{uuid}",
    tag = "Transaction Types",
    params(("uuid" = Uuid, Path, description = "Transaction type UUID")),
    request_body = UpdateTransactionTypeDto,
    responses(
        (status = 200, description = "Transaction type updated", body = TransactionTypeResponse),
        (status = 404, description = "Transaction type not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTransactionTypeDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "transaction_types", "update", None).await?;
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = transaction_types::Entity::find_by_id(id)
        .filter(transaction_types::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction type not found".to_string()))?;

    let old_json = json!({"uuid": current.uuid, "name": current.name, "code": current.code});
    let mut active: transaction_types::ActiveModel = current.into();
    if let Some(name) = &body.name {
        active.name = Set(name.clone());
    }
    if let Some(code) = &body.code {
        active.code = Set(code.to_uppercase());
    }
    if let Some(description) = &body.description {
        active.description = Set(Some(description.clone()));
    }
    active.updated_at = Set(Utc::now());

    let updated = active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "UPDATE",
        "transaction_types",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "name": updated.name, "code": updated.code})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete a transaction type
#[utoipa::path(
    delete,
    path = "/api/v1/transaction-types/{uuid}",
    tag = "Transaction Types",
    params(("uuid" = Uuid, Path, description = "Transaction type UUID")),
    responses(
        (status = 204, description = "Transaction type deleted"),
        (status = 404, description = "Transaction type not found"),
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "transaction_types", "delete", None).await?;
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = transaction_types::Entity::find_by_id(id)
        .filter(transaction_types::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction type not found".to_string()))?;

    let mut active: transaction_types::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "transaction_types",
        id,
        Some(json!({"uuid": current.uuid, "name": current.name, "code": current.code})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
