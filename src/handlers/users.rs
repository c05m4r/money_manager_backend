// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    entities::users,
    errors::AppError,
    models::{
        pagination::PaginatedResponse,
        users::{CreateUserDto, UpdateUserDto, UserListQuery, UserResponse},
    },
    services::{
        audit::write_audit_log,
        auth::{generate_email_verification_token, hash_password},
        authorization::{is_own_scoped_role, ROLE_USER},
        email::send_audit_email,
        onboarding::seed_default_data_for_user,
    },
    AppState,
};

use super::utils::{authorize_action, paginated, request_ip, validate_dto, AuthorizedUser};

fn to_response(model: users::Model) -> UserResponse {
    UserResponse {
        uuid: model.uuid,
        username: model.username,
        email: model.email,
        role: model.role,
        description: model.description,
        is_active: model.is_active,
        email_verified_at: model.email_verified_at,
        last_login_at: model.last_login_at,
        created_at: model.created_at,
        updated_at: model.updated_at,
        deleted_at: model.deleted_at,
    }
}

/// List users (paginated)
#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    params(UserListQuery),
    responses(
        (status = 200, description = "Paginated list of users", body = inline(PaginatedResponse<UserResponse>)),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    query: web::Query<UserListQuery>,
) -> Result<HttpResponse, AppError> {
    authorize_action(&state, &auth, "users", "list", Some(auth.uuid)).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);

    let mut select = users::Entity::find().filter(users::Column::DeletedAt.is_null());

    if is_own_scoped_role(&auth.role) {
        select = select.filter(users::Column::Uuid.eq(auth.uuid));
    }

    if let Some(search) = &query.search {
        select = select.filter(
            users::Column::Username
                .contains(search)
                .or(users::Column::Email.contains(search)),
        );
    }
    if let Some(is_active) = query.is_active {
        select = select.filter(users::Column::IsActive.eq(is_active));
    }
    if let Some(email_verified) = query.email_verified {
        select = if email_verified {
            select.filter(users::Column::EmailVerifiedAt.is_not_null())
        } else {
            select.filter(users::Column::EmailVerifiedAt.is_null())
        };
    }

    let paginator = select.paginate(&state.db, per_page);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page - 1).await?;
    let data = rows.into_iter().map(to_response).collect::<Vec<_>>();

    let response: PaginatedResponse<UserResponse> =
        paginated(data, total, page, per_page, "/api/v1/users");
    Ok(HttpResponse::Ok().json(response))
}

/// Get a user by UUID
#[utoipa::path(
    get,
    path = "/api/v1/users/{uuid}",
    tag = "Users",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_one(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user = users::Entity::find_by_id(path.into_inner())
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    authorize_action(&state, &auth, "users", "get", Some(user.uuid)).await?;

    Ok(HttpResponse::Ok().json(to_response(user)))
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "Users",
    request_body = CreateUserDto,
    responses(
        (status = 201, description = "User created", body = UserResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already exists"),
    ),
    security(()),
)]
pub async fn create(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateUserDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;

    if users::Entity::find()
        .filter(users::Column::Email.eq(body.email.clone()))
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    let now = Utc::now();
    let model = users::ActiveModel {
        uuid: Set(Uuid::now_v7()),
        username: Set(body.username.clone()),
        email: Set(body.email.clone()),
        password_hash: Set(hash_password(&body.password)?),
        role: Set(ROLE_USER.to_string()),
        description: Set(None),
        is_active: Set(true),
        email_verified_at: Set(None),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await?;

    let actor_uuid = model.uuid;

    seed_default_data_for_user(&state.db, model.uuid).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "INSERT",
        "users",
        model.uuid,
        None,
        Some(json!({"uuid": model.uuid, "email": model.email})),
        request_ip(&req),
    )
    .await?;

    let verify_token = generate_email_verification_token(model.uuid, &state.config)?;
    let verify_target = match &state.config.email_verification_url_base {
        Some(url_base) => format!("{url_base}?token={verify_token}"),
        None => verify_token,
    };

    let email_body = match &state.config.email_verification_url_base {
        Some(_) => format!(
            "Welcome to Money Manager, {}.\n\nPlease verify your email by clicking this link:\n{}\n\nThis link expires in {} minutes.",
            model.username,
            verify_target,
            state.config.email_verification_exp_minutes,
        ),
        None => format!(
            "Welcome to Money Manager, {}.\n\nPlease verify your email with this token:\n{}\n\nThis token expires in {} minutes.",
            model.username,
            verify_target,
            state.config.email_verification_exp_minutes,
        ),
    };

    if let Err(error) = send_audit_email(
        &state.config,
        &model.email,
        "[Action Required] Verify your email",
        &email_body,
    )
    .await
    {
        log::warn!(
            "Failed to send verification email to {}: {}",
            model.email,
            error
        );
    }

    Ok(HttpResponse::Created().json(to_response(model)))
}

/// Update an existing user
#[utoipa::path(
    patch,
    path = "/api/v1/users/{uuid}",
    tag = "Users",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    request_body = UpdateUserDto,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<String>,
    raw_body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let actor_uuid = auth.uuid;

    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| AppError::BadRequest(format!("UUID parsing failed: {e}")))?;

    let body: UpdateUserDto = serde_json::from_slice(&raw_body)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON body: {e}")))?;

    validate_dto(&body)?;

    let current = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    authorize_action(&state, &auth, "users", "update", Some(current.uuid)).await?;

    let old_json = json!({"uuid": current.uuid, "username": current.username, "email": current.email, "is_active": current.is_active});

    let mut active: users::ActiveModel = current.into();
    if let Some(username) = &body.username {
        active.username = Set(username.clone());
    }
    if let Some(email) = &body.email {
        active.email = Set(email.clone());
    }
    if let Some(is_active) = body.is_active {
        active.is_active = Set(is_active);
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
        "users",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "username": updated.username, "email": updated.email, "is_active": updated.is_active})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(to_response(updated)))
}

/// Soft-delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{uuid}",
    tag = "Users",
    params(("uuid" = Uuid, Path, description = "User UUID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn delete(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let actor_uuid = auth.uuid;
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|e| AppError::BadRequest(format!("UUID parsing failed: {e}")))?;

    let current = users::Entity::find_by_id(id)
        .filter(users::Column::DeletedAt.is_null())
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    authorize_action(&state, &auth, "users", "delete", Some(current.uuid)).await?;

    let mut active: users::ActiveModel = current.clone().into();
    active.deleted_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        actor_uuid,
        "DELETE",
        "users",
        id,
        Some(json!({"uuid": current.uuid, "username": current.username, "email": current.email})),
        None,
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
