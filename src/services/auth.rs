// Copyright (C) 2026 Marcos Gabriel Miller
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::AppConfig, errors::AppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetClaims {
    pub sub: String,
    pub purpose: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationClaims {
    pub sub: String,
    pub purpose: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|value| value.to_string())
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_token(user_uuid: Uuid, role: &str, config: &AppConfig) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::hours(config.jwt_exp_hours)).timestamp() as usize;
    let claims = Claims {
        sub: user_uuid.to_string(),
        role: role.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub fn decode_token(token: &str, config: &AppConfig) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub fn generate_password_reset_token(
    user_uuid: Uuid,
    config: &AppConfig,
) -> Result<String, AppError> {
    let exp =
        (Utc::now() + Duration::minutes(config.password_reset_exp_minutes)).timestamp() as usize;
    let claims = PasswordResetClaims {
        sub: user_uuid.to_string(),
        purpose: "password_reset".to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub fn decode_password_reset_token(
    token: &str,
    config: &AppConfig,
) -> Result<PasswordResetClaims, AppError> {
    let claims = decode::<PasswordResetClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)?;

    if claims.purpose != "password_reset" {
        return Err(AppError::Unauthorized);
    }

    Ok(claims)
}

pub fn generate_email_verification_token(
    user_uuid: Uuid,
    config: &AppConfig,
) -> Result<String, AppError> {
    let exp =
        (Utc::now() + Duration::minutes(config.email_verification_exp_minutes)).timestamp()
            as usize;
    let claims = EmailVerificationClaims {
        sub: user_uuid.to_string(),
        purpose: "email_verification".to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub fn decode_email_verification_token(
    token: &str,
    config: &AppConfig,
) -> Result<EmailVerificationClaims, AppError> {
    let claims = decode::<EmailVerificationClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)?;

    if claims.purpose != "email_verification" {
        return Err(AppError::Unauthorized);
    }

    Ok(claims)
}

pub fn bearer_token_from_header(header: Option<&str>) -> Result<&str, AppError> {
    let value = header.ok_or(AppError::Unauthorized)?;
    value
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)
}
