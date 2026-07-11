// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entities::accounts,
    entities::transactions,
    errors::AppError,
    models::{
        pagination::PaginatedResponse,
        transactions::{
            CreateTransactionDto, TransactionListQuery, TransactionResponse, UpdateTransactionDto,
        },
    },
    services::{audit::write_audit_log, authorization::is_own_scoped_role},
    AppState,
};

use super::utils::{paginated, request_ip, validate_dto, AuthorizedUser};

fn end_of_day() -> NaiveTime {
    NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999).expect("valid end of day")
}

fn parse_amount(value: &str) -> Result<Decimal, AppError> {
    value
        .parse::<Decimal>()
        .map_err(|_| AppError::BadRequest("amount must be a decimal number".to_string()))
}

fn to_response(model: transactions::Model) -> TransactionResponse {
    TransactionResponse {
        uuid: model.uuid,
        amount: model.amount.to_string(),
        description: model.description,
        transaction_date: model.transaction_date,
        type_uuid: model.type_uuid,
        category_uuid: model.category_uuid,
        account_uuid: model.account_uuid,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List transactions (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/transactions",
    tag = "Transactions",
    params(TransactionListQuery),
    responses(
        (status = 200, description = "Paginated list of transactions", body = inline(PaginatedResponse<TransactionResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<TransactionListQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = transactions::Entity::find().filter(transactions::Column::DeletedAt.is_null());

    if let Some(account_uuid) = query.account_uuid {
        select = select.filter(transactions::Column::AccountUuid.eq(account_uuid));
    }
    if let Some(category_uuid) = query.category_uuid {
        select = select.filter(transactions::Column::CategoryUuid.eq(category_uuid));
    }
    if let Some(type_uuid) = query.type_uuid {
        select = select.filter(transactions::Column::TypeUuid.eq(type_uuid));
    }
    if let Some(search) = &query.search {
        select = select.filter(transactions::Column::Description.contains(search));
    }
    if let Some(amount_min) = &query.amount_min {
        select = select.filter(transactions::Column::Amount.gte(parse_amount(amount_min)?));
    }
    if let Some(amount_max) = &query.amount_max {
        select = select.filter(transactions::Column::Amount.lte(parse_amount(amount_max)?));
    }
    if let Some(date_from) = query.date_from {
        let from = DateTime::<Utc>::from_naive_utc_and_offset(date_from.and_time(NaiveTime::MIN), Utc);
        select = select.filter(transactions::Column::TransactionDate.gte(from));
    }
    if let Some(date_to) = query.date_to {
        let to = DateTime::<Utc>::from_naive_utc_and_offset(date_to.and_time(end_of_day()), Utc);
        select = select.filter(transactions::Column::TransactionDate.lte(to));
    }
    if is_own_scoped_role(&auth.role) {
        let account_ids = accounts::Entity::find()
            .filter(accounts::Column::DeletedAt.is_null())
            .filter(accounts::Column::UserUuid.eq(auth.uuid))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|account| account.uuid)
            .collect::<Vec<_>>();

        if account_ids.is_empty() {
            let response: PaginatedResponse<TransactionResponse> =
                paginated(Vec::new(), 0, page, per_page, "/api/v1/transactions");
            return Ok(HttpResponse::Ok().json(response));
        }

        select = select.filter(transactions::Column::AccountUuid.is_in(account_ids));
    } else if let Some(user_uuid) = query.user_uuid {
        let account_ids = accounts::Entity::find()
            .filter(accounts::Column::DeletedAt.is_null())
            .filter(accounts::Column::UserUuid.eq(user_uuid))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|account| account.uuid)
            .collect::<Vec<_>>();

        if account_ids.is_empty() {
            let response: PaginatedResponse<TransactionResponse> =
                paginated(Vec::new(), 0, page, per_page, "/api/v1/transactions");
            return Ok(HttpResponse::Ok().json(response));
        }

        select = select.filter(transactions::Column::AccountUuid.is_in(account_ids));
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<TransactionResponse> =
        paginated(data, total, page, per_page, "/api/v1/transactions");
    Ok(HttpResponse::Ok().json(response))
}

/// Get a transaction by UUID
#[utoipa::path(
    get,
    path = "/api/v1/transactions/{uuid}",
    tag = "Transactions",
    params(("uuid" = Uuid, Path, description = "Transaction UUID")),
    responses(
        (status = 200, description = "Transaction found", body = TransactionResponse),
        (status = 404, description = "Transaction not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    _auth: AuthorizedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let row = transactions::Entity::find_by_id(path.into_inner())
        .filter(transactions::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction not found".to_string()))?;

    Ok(HttpResponse::Ok().json(to_response(row)))
}

/// Create a new transaction
#[utoipa::path(
    post,
    path = "/api/v1/transactions",
    tag = "Transactions",
    request_body = CreateTransactionDto,
    responses(
        (status = 201, description = "Transaction created", body = TransactionResponse),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<CreateTransactionDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    let actor_uuid = auth.uuid;
    let now = Utc::now();

    let row = transactions::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        amount: Set(parse_amount(&body.amount)?),
        description: Set(body.description.clone()),
        transaction_date: Set(body.transaction_date),
        type_uuid: Set(body.type_uuid),
        category_uuid: Set(body.category_uuid),
        account_uuid: Set(body.account_uuid),
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
        "transactions",
        row.uuid,
        None,
        Some(json!({"uuid": row.uuid, "amount": row.amount.to_string()})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Created().json(to_response(row)))
}

/// Update a transaction
#[utoipa::path(
    patch,
    path = "/api/v1/transactions/{uuid}",
    tag = "Transactions",
    params(("uuid" = Uuid, Path, description = "Transaction UUID")),
    request_body = UpdateTransactionDto,
    responses(
        (status = 200, description = "Transaction updated", body = TransactionResponse),
        (status = 404, description = "Transaction not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateTransactionDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = transactions::Entity::find_by_id(id)
        .filter(transactions::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction not found".to_string()))?;

    let old_json = json!({"uuid": current.uuid, "amount": current.amount.to_string(), "description": current.description});
    let mut active: transactions::ActiveModel = current.into();
    if let Some(amount) = &body.amount {
        active.amount = Set(parse_amount(amount)?);
    }
    if let Some(description) = &body.description {
        active.description = Set(Some(description.clone()));
    }
    if let Some(transaction_date) = body.transaction_date {
        active.transaction_date = Set(transaction_date);
    }
    if let Some(type_uuid) = body.type_uuid {
        active.type_uuid = Set(type_uuid);
    }
    active.category_uuid = Set(body.category_uuid);
    active.account_uuid = Set(body.account_uuid);
    active.updated_at = Set(Utc::now());

    let updated = active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "UPDATE",
        "transactions",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "amount": updated.amount.to_string(), "description": updated.description})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete a transaction
#[utoipa::path(
    delete,
    path = "/api/v1/transactions/{uuid}",
    tag = "Transactions",
    params(("uuid" = Uuid, Path, description = "Transaction UUID")),
    responses(
        (status = 204, description = "Transaction deleted"),
        (status = 404, description = "Transaction not found"),
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = transactions::Entity::find_by_id(id)
        .filter(transactions::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Transaction not found".to_string()))?;

    let mut active: transactions::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "transactions",
        id,
        Some(json!({"uuid": current.uuid, "amount": current.amount.to_string(), "description": current.description})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
