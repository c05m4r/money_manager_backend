// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entities::categories,
    errors::AppError,
    models::{
        categories::{CategoryListQuery, CategoryResponse, CreateCategoryDto, UpdateCategoryDto},
        pagination::PaginatedResponse,
    },
    services::{audit::write_audit_log, authorization::is_own_scoped_role},
    AppState,
};

use super::utils::{authorize_action, paginated, request_ip, validate_dto, AuthorizedUser};

fn to_response(model: categories::Model) -> CategoryResponse {
    CategoryResponse {
        uuid: model.uuid,
        user_uuid: model.user_uuid,
        name: model.name,
        description: model.description,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List categories (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/categories",
    tag = "Categories",
    params(CategoryListQuery),
    responses(
        (status = 200, description = "Paginated list of categories", body = inline(PaginatedResponse<CategoryResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<CategoryListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "categories", "list", Some(auth.uuid)).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = categories::Entity::find().filter(categories::Column::DeletedAt.is_null());

    if is_own_scoped_role(&auth.role) {
        select = select.filter(categories::Column::UserUuid.eq(auth.uuid));
    } else if let Some(user_uuid) = query.user_uuid {
        select = select.filter(categories::Column::UserUuid.eq(user_uuid));
    }
    if let Some(search) = &query.search {
        select = select.filter(categories::Column::Name.contains(search));
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<CategoryResponse> =
        paginated(data, total, page, per_page, "/api/v1/categories");
    Ok(HttpResponse::Ok().json(response))
}

/// Get a category by UUID
#[utoipa::path(
    get,
    path = "/api/v1/categories/{uuid}",
    tag = "Categories",
    params(("uuid" = Uuid, Path, description = "Category UUID")),
    responses(
        (status = 200, description = "Category found", body = CategoryResponse),
        (status = 404, description = "Category not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let row = categories::Entity::find_by_id(path.into_inner())
        .filter(categories::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Category not found".to_string()))?;

    authorize_action(&state, &auth, "categories", "get", Some(row.user_uuid)).await?;

    Ok(HttpResponse::Ok().json(to_response(row)))
}

/// Create a new category
#[utoipa::path(
    post,
    path = "/api/v1/categories",
    tag = "Categories",
    request_body = CreateCategoryDto,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn create(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<CreateCategoryDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    authorize_action(&state, &auth, "categories", "create", Some(body.user_uuid)).await?;
    let actor_uuid = auth.uuid;

    let now = Utc::now();
    let row = categories::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        user_uuid: Set(body.user_uuid),
        name: Set(body.name.clone()),
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
        "categories",
        row.uuid,
        None,
        Some(json!({"uuid": row.uuid, "name": row.name})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Created().json(to_response(row)))
}

/// Update a category
#[utoipa::path(
    patch,
    path = "/api/v1/categories/{uuid}",
    tag = "Categories",
    params(("uuid" = Uuid, Path, description = "Category UUID")),
    request_body = UpdateCategoryDto,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
        (status = 404, description = "Category not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateCategoryDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    let actor_uuid = auth.uuid;
    let id = path.into_inner();

    let current = categories::Entity::find_by_id(id)
        .filter(categories::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Category not found".to_string()))?;

    authorize_action(&state, &auth, "categories", "update", Some(current.user_uuid)).await?;

    let old_json = json!({"uuid": current.uuid, "name": current.name});
    let mut active: categories::ActiveModel = current.into();
    if let Some(name) = &body.name {
        active.name = Set(name.clone());
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
        "categories",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "name": updated.name})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete a category
#[utoipa::path(
    delete,
    path = "/api/v1/categories/{uuid}",
    tag = "Categories",
    params(("uuid" = Uuid, Path, description = "Category UUID")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found"),
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

    let current = categories::Entity::find_by_id(id)
        .filter(categories::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("Category not found".to_string()))?;

    authorize_action(&state, &auth, "categories", "delete", Some(current.user_uuid)).await?;

    let mut active: categories::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "categories",
        id,
        Some(json!({"uuid": current.uuid, "name": current.name})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
