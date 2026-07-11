// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
};
use serde_json::json;

use crate::{
    entities::{audit_logs, users},
    errors::AppError,
    models::users::{
        ChangePasswordDto, ForgotPasswordDto, LoginDto, LoginResponse, ResetPasswordDto,
        UserResponse, VerifyEmailQuery,
    },
    services::{
        audit::write_audit_log,
        auth::{
            decode_email_verification_token, decode_password_reset_token,
            generate_email_verification_token, generate_password_reset_token, generate_token,
            hash_password, verify_password,
        },
        authorization::normalize_role,
        email::send_audit_email,
    },
    AppState,
};

use super::utils::{request_ip, validate_dto, AuthorizedUser};

async fn send_security_email(
    state: &web::Data<AppState>,
    user: &users::Model,
    subject: &str,
    body: &str,
) {
    let mut recipients = vec![user.email.clone()];
    if let Some(extra_recipient) = &state.config.audit_email_to {
        if !recipients.iter().any(|value| value == extra_recipient) {
            recipients.push(extra_recipient.clone());
        }
    }

    for recipient in recipients {
        if let Err(error) = send_audit_email(&state.config, &recipient, subject, body).await {
            log::warn!("Failed to send audit email to {recipient}: {error}");
        }
    }
}

async fn send_email_verification_message(state: &web::Data<AppState>, user: &users::Model) {
    let Ok(token) = generate_email_verification_token(user.uuid, &state.config) else {
        log::warn!(
            "Failed to generate email verification token for user {}",
            user.uuid
        );
        return;
    };

    let verify_target = match &state.config.email_verification_url_base {
        Some(url_base) => format!("{url_base}?token={token}"),
        None => token.clone(),
    };

    let body = match &state.config.email_verification_url_base {
        Some(_) => format!(
            "Please verify your email address.\n\nUser: {}\nVerification link: {}\nExpires in: {} minutes",
            user.username,
            verify_target,
            state.config.email_verification_exp_minutes,
        ),
        None => format!(
            "Please verify your email address.\n\nUser: {}\nVerification token: {}\nExpires in: {} minutes",
            user.username,
            verify_target,
            state.config.email_verification_exp_minutes,
        ),
    };

    if let Err(error) = send_audit_email(
        &state.config,
        &user.email,
        "[Action Required] Verify your email",
        &body,
    )
    .await
    {
        log::warn!(
            "Failed to send email verification message to {}: {}",
            user.email,
            error
        );
    }
}

/// Login with username and password
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body(
        content = LoginDto,
        example = json!({
            "username": "admin",
            "password": "ContraseniaSegura2026!"
        })
    ),
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    security(()),
)]
pub async fn login(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<LoginDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;
    body.0.validate_identifier()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut query = users::Entity::find()
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true));

    if let Some(ref username) = body.username {
        query = query.filter(users::Column::Username.eq(username.clone()));
    } else if let Some(ref email) = body.email {
        query = query.filter(users::Column::Email.eq(email.clone()));
    }

    let user = query
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let valid = verify_password(&body.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let ip = request_ip(&req);
    let has_login_from_ip = audit_logs::Entity::find()
        .filter(audit_logs::Column::UserUuid.eq(user.uuid))
        .filter(audit_logs::Column::Action.eq("LOGIN"))
        .filter(audit_logs::Column::IpAddress.eq(ip.clone()))
        .count(&state.db)
        .await?
        > 0;

    let mut active: users::ActiveModel = user.clone().into();
    active.last_login_at = Set(Some(Utc::now()));
    active.updated_at = Set(Utc::now());
    active.update(&state.db).await?;

    let role = normalize_role(&user.role);
    let token = generate_token(user.uuid, &role, &state.config)?;
    write_audit_log(
        &state.db,
        user.uuid,
        "LOGIN",
        "users",
        user.uuid,
        None,
        Some(json!({"username": user.username})),
        ip.clone(),
    )
    .await?;

    if !has_login_from_ip {
        let body = format!(
            "An unrecognized IP address was used to sign in.\n\nUser: {}\nIP: {}\nDate (UTC): {}",
            user.username,
            ip,
            Utc::now().to_rfc3339(),
        );
        send_security_email(
            &state,
            &user,
            "[Audit] Login from unrecognized IP address",
            &body,
        )
        .await;
    }

    let response = UserResponse {
        uuid: user.uuid,
        username: user.username,
        email: user.email,
        role,
        description: user.description,
        is_active: user.is_active,
        email_verified_at: user.email_verified_at,
        last_login_at: Some(Utc::now()),
        created_at: user.created_at,
        updated_at: Utc::now(),
        deleted_at: user.deleted_at,
    };

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        user: response,
    }))
}

/// Logout current session
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "Auth",
    responses(
        (status = 200, description = "Logged out successfully"),
    ),
    security(()),
)]
pub async fn logout() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(json!({"message": "Logged out"})))
}

/// Request password reset instructions
#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password",
    tag = "Auth",
    request_body(
        content = ForgotPasswordDto,
        example = json!({
            "email": "admin@moneymanager.com"
        })
    ),
    responses(
        (status = 200, description = "Password reset instructions sent if the account exists"),
    ),
    security(())
)]
pub async fn forgot_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<ForgotPasswordDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;

    let user = users::Entity::find()
        .filter(users::Column::Email.eq(body.email.clone()))
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?;

    if let Some(user) = user {
        let token = generate_password_reset_token(user.uuid, &state.config)?;
        let reset_target = match &state.config.password_reset_url_base {
            Some(url_base) => format!("{url_base}?token={token}"),
            None => token.clone(),
        };

        let body = match &state.config.password_reset_url_base {
            Some(_) => format!(
                "A password reset was requested for your account.\n\nUser: {}\nReset link: {}\nExpires in: {} minutes",
                user.username,
                reset_target,
                state.config.password_reset_exp_minutes,
            ),
            None => format!(
                "A password reset was requested for your account.\n\nUser: {}\nReset token: {}\nExpires in: {} minutes",
                user.username,
                reset_target,
                state.config.password_reset_exp_minutes,
            ),
        };

        if let Err(error) = send_audit_email(
            &state.config,
            &user.email,
            "[Security] Password reset requested",
            &body,
        )
        .await
        {
            log::warn!("Failed to send password reset email to {}: {}", user.email, error);
        }

        write_audit_log(
            &state.db,
            user.uuid,
            "FORGOT_PASSWORD_REQUEST",
            "users",
            user.uuid,
            None,
            Some(json!({"email": user.email})),
            request_ip(&req),
        )
        .await?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "If the account exists, password reset instructions have been sent"
    })))
}

/// Reset password with token from forgot-password flow
#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password",
    tag = "Auth",
    request_body(
        content = ResetPasswordDto,
        example = json!({
            "token": "<password-reset-token>",
            "new_password": "ContraseniaNuevaSegura2026!"
        })
    ),
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 401, description = "Invalid or expired reset token"),
    ),
    security(())
)]
pub async fn reset_password(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<ResetPasswordDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;

    let claims = decode_password_reset_token(&body.token, &state.config)?;
    let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let user = users::Entity::find_by_id(user_uuid)
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let new_password_matches_current = verify_password(&body.new_password, &user.password_hash)?;
    if new_password_matches_current {
        return Err(AppError::BadRequest(
            "New password cannot be the same as current password".to_string(),
        ));
    }

    let now = Utc::now();
    let mut active: users::ActiveModel = user.clone().into();
    active.password_hash = Set(hash_password(&body.new_password)?);
    active.updated_at = Set(now);
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        user.uuid,
        "PASSWORD_RESET",
        "users",
        user.uuid,
        None,
        Some(json!({"password_reset_at": now})),
        request_ip(&req),
    )
    .await?;

    let body = format!(
        "A password reset was completed for your account.\n\nUser: {}\nDate (UTC): {}",
        user.username,
        now.to_rfc3339(),
    );
    send_security_email(&state, &user, "[Audit] Password reset completed", &body).await;

    Ok(HttpResponse::Ok().json(json!({"message": "Password reset successfully"})))
}

/// Change current user password
#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password",
    tag = "Auth",
    request_body = ChangePasswordDto,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn change_password(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
    body: web::Json<ChangePasswordDto>,
) -> Result<HttpResponse, AppError> {
    validate_dto(&body.0)?;

    let user = users::Entity::find_by_id(auth.uuid)
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let is_current_password_valid = verify_password(&body.current_password, &user.password_hash)?;
    if !is_current_password_valid {
        return Err(AppError::Unauthorized);
    }

    let new_password_matches_current = verify_password(&body.new_password, &user.password_hash)?;
    if new_password_matches_current {
        return Err(AppError::BadRequest(
            "New password cannot be the same as current password".to_string(),
        ));
    }

    let now = Utc::now();
    let mut active: users::ActiveModel = user.clone().into();
    active.password_hash = Set(hash_password(&body.new_password)?);
    active.updated_at = Set(now);
    active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        user.uuid,
        "PASSWORD_CHANGE",
        "users",
        user.uuid,
        None,
        Some(json!({"password_changed_at": now})),
        request_ip(&req),
    )
    .await?;

    let body = format!(
        "A password change was made to your account.\n\nUser: {}\nDate (UTC): {}",
        user.username,
        now.to_rfc3339(),
    );
    send_security_email(
        &state,
        &user,
        "[Audit] Password changed",
        &body,
    )
    .await;

    Ok(HttpResponse::Ok().json(json!({"message": "Password changed successfully"})))
}

/// Verify email with token from verification link
#[utoipa::path(
    get,
    path = "/api/v1/auth/verify-email",
    tag = "Auth",
    params(VerifyEmailQuery),
    responses(
        (status = 200, description = "Email verified successfully"),
        (status = 401, description = "Unauthorized"),
    )
    ,
    security(())
)]
pub async fn verify_email(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<VerifyEmailQuery>,
) -> Result<HttpResponse, AppError> {
    let claims = decode_email_verification_token(&query.token, &state.config)?;
    let user_uuid = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let user = users::Entity::find_by_id(user_uuid)
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user.email_verified_at.is_some() {
        return Ok(HttpResponse::Ok().json(json!({"message": "Email already verified"})));
    }

    let now = Utc::now();
    let old_json = json!({"uuid": user.uuid, "email_verified_at": user.email_verified_at});

    let mut active: users::ActiveModel = user.clone().into();
    active.email_verified_at = Set(Some(now));
    active.updated_at = Set(now);

    let updated = active.update(&state.db).await?;

    write_audit_log(
        &state.db,
        updated.uuid,
        "EMAIL_VERIFY",
        "users",
        updated.uuid,
        Some(old_json),
        Some(json!({"uuid": updated.uuid, "email_verified_at": updated.email_verified_at})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({"message": "Email verified successfully"})))
}

/// Resend verification email to current authenticated user
#[utoipa::path(
    post,
    path = "/api/v1/auth/resend-verification-email",
    tag = "Auth",
    responses(
        (status = 200, description = "Verification email sent"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn resend_verification_email(
    state: web::Data<AppState>,
    auth: AuthorizedUser,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user = users::Entity::find_by_id(auth.uuid)
        .filter(users::Column::DeletedAt.is_null())
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user.email_verified_at.is_some() {
        return Ok(HttpResponse::Ok().json(json!({"message": "Email already verified"})));
    }

    send_email_verification_message(&state, &user).await;

    write_audit_log(
        &state.db,
        auth.uuid,
        "EMAIL_VERIFICATION_SENT",
        "users",
        user.uuid,
        None,
        Some(json!({"uuid": user.uuid, "email": user.email})),
        request_ip(&req),
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Verification email sent"
    })))
}
