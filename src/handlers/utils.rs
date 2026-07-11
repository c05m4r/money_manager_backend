// Copyright (C) 2026 Marcos Gabriel Miller
use actix_web::{
    dev::Payload,
    http::header,
    web, FromRequest, HttpRequest,
};
use serde::Serialize;
use std::future::{ready, Ready};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    errors::AppError,
    models::pagination::{build_links, Meta, PaginatedResponse},
    services::authorization,
    services::auth::{bearer_token_from_header, decode_token},
    AppState,
};

pub fn validate_dto<T: Validate>(dto: &T) -> Result<(), AppError> {
    dto.validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))
}

pub fn request_ip(req: &HttpRequest) -> String {
    if let Some(forwarded) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    {
        if let Some(first_ip) = forwarded.split(',').next() {
            let ip = first_ip.trim();
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    if let Some(real_ip) = req
        .headers()
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
    {
        let ip = real_ip.trim();
        if !ip.is_empty() {
            return ip.to_string();
        }
    }

    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Clone, Debug)]
pub struct AuthorizedUser {
    pub uuid: Uuid,
    pub role: String,
}

impl FromRequest for AuthorizedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let Some(state) = req.app_data::<web::Data<AppState>>() else {
            return ready(Err(AppError::InternalServerError(
                "App state is not configured".to_string(),
            )));
        };

        let header_value = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let result = bearer_token_from_header(header_value)
            .and_then(|token| decode_token(token, &state.config))
            .and_then(|claims| {
                let uuid = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
                let role = authorization::normalize_role(&claims.role);
                Ok(AuthorizedUser { uuid, role })
            });

        ready(result)
    }
}

pub async fn authorize_action(
    state: &web::Data<AppState>,
    auth: &AuthorizedUser,
    resource: &str,
    action: &str,
    owner_uuid: Option<Uuid>,
) -> Result<(), AppError> {
    authorization::authorize(
        &state.enforcer,
        &auth.role,
        auth.uuid,
        resource,
        action,
        owner_uuid,
    )
    .await
}

pub fn paginated<T: Serialize + ToSchema>(
    data: Vec<T>,
    total_records: u64,
    page: u64,
    per_page: u64,
    base_path: &str,
) -> PaginatedResponse<T> {
    let total_pages = ((total_records + per_page - 1) / per_page).max(1);
    let links = build_links(base_path, page.min(total_pages), per_page, total_pages);
    PaginatedResponse {
        data,
        meta: Meta {
            total_records,
            current_page: page,
            total_pages,
            per_page,
        },
        links,
    }
}
