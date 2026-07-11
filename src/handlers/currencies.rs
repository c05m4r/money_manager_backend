// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, PaginatorTrait,
    QueryFilter,
};
use serde_json::json;

use crate::{
    entities::currencies,
    errors::AppError,
    models::{
        currencies::{CreateCurrencyDto, CurrencyListQuery, CurrencyResponse, UpdateCurrencyDto},
        pagination::PaginatedResponse,
    },
    services::audit::write_audit_log,
    AppState,
};

use super::utils::{authorize_action, paginated, request_ip, validate_dto, AuthorizedUser};

fn to_response(model: currencies::Model) -> CurrencyResponse {
    CurrencyResponse {
        code: model.code,
        name: model.name,
        symbol: model.symbol,
        description: model.description,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List currencies (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/currencies",
    tag = "Currencies",
    params(CurrencyListQuery),
    responses(
        (status = 200, description = "Paginated list of currencies", body = inline(PaginatedResponse<CurrencyResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<CurrencyListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "currencies", "list", None).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = currencies::Entity::find().filter(currencies::Column::DeletedAt.is_null());
    if let Some(search) = &query.search {
        select = select.filter(
            Condition::any()
                .add(currencies::Column::Code.contains(search))
                .add(currencies::Column::Name.contains(search))
                .add(currencies::Column::Symbol.contains(search)),
        );
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<CurrencyResponse> =
        paginated(data, total, page, per_page, "/api/v1/currencies");
    Ok(HttpResponse::Ok().json(response))
}

/// Get a currency by code
#[utoipa::path(
    get,
    path = "/api/v1/currencies/{code}",
    tag = "Currencies",
    params(("code" = String, Path, description = "Currency ISO code (e.g. USD)")),
    responses(
        (status = 200, description = "Currency found", body = CurrencyResponse),
        (status = 404, description = "Currency not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "currencies", "get", None).await?;

    let code = path.into_inner().to_uppercase();
    let row = currencies::Entity::find_by_id(code)
        .filter(currencies::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Currency not found".to_string()))?;
    Ok(HttpResponse::Ok().json(to_response(row)))
}

/// Create a new currency
#[utoipa::path(
    post,
    path = "/api/v1/currencies",
    tag = "Currencies",
    request_body = CreateCurrencyDto,
    responses(
        (status = 201, description = "Currency created", body = CurrencyResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Currency already exists"),
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<CreateCurrencyDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "currencies", "create", None).await?;
    let actor_uuid = auth.uuid;
    let code = body.code.to_uppercase();

    if currencies::Entity::find_by_id(code.clone())
        .one(&state.db)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Currency already exists".to_string()));
    }

    let now = Utc::now();
    let row = currencies::ActiveModel {
        code: Set(code.clone()),
        name: Set(body.name.clone()),
        symbol: Set(body.symbol.clone()),
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
        "currencies",
        uuid::Uuid::now_v7(),
        None,
        Some(json!({"code": row.code})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Created().json(to_response(row)))
}

/// Update a currency
#[utoipa::path(
    patch,
    path = "/api/v1/currencies/{code}",
    tag = "Currencies",
    params(("code" = String, Path, description = "Currency ISO code")),
    request_body = UpdateCurrencyDto,
    responses(
        (status = 200, description = "Currency updated", body = CurrencyResponse),
        (status = 404, description = "Currency not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateCurrencyDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "currencies", "update", None).await?;
    let actor_uuid = auth.uuid;
    let code = path.into_inner().to_uppercase();

    let current = currencies::Entity::find_by_id(code)
        .filter(currencies::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Currency not found".to_string()))?;

    let old_json = json!({"code": current.code, "name": current.name, "symbol": current.symbol});

    let mut active: currencies::ActiveModel = current.into();
    if let Some(name) = &body.name {
        active.name = Set(name.clone());
    }
    if let Some(symbol) = &body.symbol {
        active.symbol = Set(symbol.clone());
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
        "currencies",
        uuid::Uuid::now_v7(),
        Some(old_json),
        Some(json!({"code": updated.code, "name": updated.name, "symbol": updated.symbol})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete a currency
#[utoipa::path(
    delete,
    path = "/api/v1/currencies/{code}",
    tag = "Currencies",
    params(("code" = String, Path, description = "Currency ISO code")),
    responses(
        (status = 204, description = "Currency deleted"),
        (status = 404, description = "Currency not found"),
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "currencies", "delete", None).await?;
    let actor_uuid = auth.uuid;
    let code = path.into_inner().to_uppercase();

    let current = currencies::Entity::find_by_id(code)
        .filter(currencies::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Currency not found".to_string()))?;

    let mut active: currencies::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "currencies",
        uuid::Uuid::now_v7(),
        Some(json!({"code": current.code, "name": current.name, "symbol": current.symbol})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
