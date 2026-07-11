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
    entities::accounts,
    errors::AppError,
    models::{
        accounts::{AccountListQuery, AccountResponse, CreateAccountDto, UpdateAccountDto},
        pagination::PaginatedResponse,
    },
    services::{audit::write_audit_log, authorization::is_own_scoped_role},
    AppState,
};

use super::utils::{authorize_action, paginated, request_ip, validate_dto, AuthorizedUser};

fn to_response(model: accounts::Model) -> AccountResponse {
    AccountResponse {
        uuid: model.uuid,
        user_uuid: model.user_uuid,
        name: model.name,
        description: model.description,
        is_default: model.is_default,
        currency_code: model.currency_code,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List accounts (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/accounts",
    tag = "Accounts",
    params(AccountListQuery),
    responses(
        (status = 200, description = "Paginated list of accounts", body = inline(PaginatedResponse<AccountResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<AccountListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "accounts", "list", Some(auth.uuid)).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = accounts::Entity::find().filter(accounts::Column::DeletedAt.is_null());

    if is_own_scoped_role(&auth.role) {
        select = select.filter(accounts::Column::UserUuid.eq(auth.uuid));
    } else if let Some(user_uuid) = query.user_uuid {
        select = select.filter(accounts::Column::UserUuid.eq(user_uuid));
    }
    if let Some(currency_code) = &query.currency_code {
        select = select.filter(accounts::Column::CurrencyCode.eq(currency_code.to_uppercase()));
    }
    if let Some(search) = &query.search {
        select = select.filter(Condition::any().add(accounts::Column::Name.contains(search)));
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<AccountResponse> =
        paginated(data, total, page, per_page, "/api/v1/accounts");
    Ok(HttpResponse::Ok().json(response))
}

/// Get an account by UUID
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{uuid}",
    tag = "Accounts",
    params(("uuid" = Uuid, Path, description = "Account UUID")),
    responses(
        (status = 200, description = "Account found", body = AccountResponse),
        (status = 404, description = "Account not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let row = accounts::Entity::find_by_id(path.into_inner())
        .filter(accounts::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Account not found".to_string()))?;

    authorize_action(&state, &auth, "accounts", "get", Some(row.user_uuid)).await?;

    Ok(HttpResponse::Ok().json(to_response(row)))
}

/// Create a new account
#[utoipa::path(
    post,
    path = "/api/v1/accounts",
    tag = "Accounts",
    request_body = CreateAccountDto,
    responses(
        (status = 201, description = "Account created", body = AccountResponse),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<CreateAccountDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "accounts", "create", Some(body.user_uuid)).await?;
    let actor_uuid = auth.uuid;

    let now = Utc::now();
    let row = accounts::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        user_uuid: Set(body.user_uuid),
        currency_code: Set(body.currency_code.to_uppercase()),
        name: Set(body.name.clone()),
        description: Set(body.description.clone()),
        is_default: Set(body.is_default.unwrap_or(false)),
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
        "accounts",
        row.uuid,
        None,
        Some(json!({"uuid": row.uuid})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Created().json(to_response(row)))
}

/// Update an account
#[utoipa::path(
    patch,
    path = "/api/v1/accounts/{uuid}",
    tag = "Accounts",
    params(("uuid" = Uuid, Path, description = "Account UUID")),
    request_body = UpdateAccountDto,
    responses(
        (status = 200, description = "Account updated", body = AccountResponse),
        (status = 404, description = "Account not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateAccountDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = accounts::Entity::find_by_id(id)
        .filter(accounts::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Account not found".to_string()))?;

    authorize_action(&state, &auth, "accounts", "update", Some(current.user_uuid)).await?;

    let old_json = json!({"uuid": current.uuid, "name": current.name, "currency_code": current.currency_code, "user_uuid": current.user_uuid});
    let mut active: accounts::ActiveModel = current.into();
    if let Some(user_uuid) = &body.user_uuid {
        active.user_uuid = Set(*user_uuid);
    }
    if let Some(name) = &body.name {
        active.name = Set(name.clone());
    }
    if let Some(code) = &body.currency_code {
        active.currency_code = Set(code.to_uppercase());
    }
    if let Some(description) = &body.description {
        active.description = Set(Some(description.clone()));
    }
    if let Some(is_default) = body.is_default {
        active.is_default = Set(is_default);
    }
    active.updated_at = Set(Utc::now());

    let updated = active.update(&state.db).await?;
    write_audit_log(
        &state.db,
        actor_uuid,
        "UPDATE",
        "accounts",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "name": updated.name, "currency_code": updated.currency_code})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete an account
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/{uuid}",
    tag = "Accounts",
    params(("uuid" = Uuid, Path, description = "Account UUID")),
    responses(
        (status = 204, description = "Account deleted"),
        (status = 404, description = "Account not found"),
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

    let current = accounts::Entity::find_by_id(id)
        .filter(accounts::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Account not found".to_string()))?;

    authorize_action(&state, &auth, "accounts", "delete", Some(current.user_uuid)).await?;

    let mut active: accounts::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "accounts",
        id,
        Some(json!({"uuid": current.uuid, "name": current.name})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
