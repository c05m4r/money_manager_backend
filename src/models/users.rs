// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::{DateTime, Utc};
use fancy_regex::Regex as FancyRegex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::{Validate, ValidationError};

const PASSWORD_POLICY_REGEX: &str =
    r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{20,}$";

static PASSWORD_POLICY: LazyLock<FancyRegex> =
    LazyLock::new(|| FancyRegex::new(PASSWORD_POLICY_REGEX).expect("invalid password policy regex"));

fn validate_password_policy(password: &str) -> Result<(), ValidationError> {
    let matches = PASSWORD_POLICY
        .is_match(password)
        .map_err(|_| ValidationError::new("password_policy_regex_error"))?;

    if matches {
        Ok(())
    } else {
        let mut error = ValidationError::new("password_policy");
        error.message = Some(
            "Password must be at least 20 characters long and include: one lowercase letter, one uppercase letter, one number, and one symbol (@$!%*?&)."
                .into(),
        );
        Err(error)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub uuid: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_password_policy"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateUserDto {
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub is_active: Option<bool>,
    #[validate(length(min = 1, max = 400))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct UserListQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginDto {
    #[schema(example = "admin")]
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[schema(example = "admin@moneymanager.com")]
    #[validate(email)]
    pub email: Option<String>,
    #[schema(example = "ContraseniaSegura2026!")]
    #[validate(custom(function = "validate_password_policy"))]
    pub password: String,
}

impl LoginDto {
    pub fn validate_identifier(&self) -> Result<(), ValidationError> {
        if self.username.is_none() && self.email.is_none() {
            let mut error = ValidationError::new("login_identifier");
            error.message = Some("Either username or email must be provided.".into());
            return Err(error);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordDto {
    #[validate(length(min = 1))]
    pub current_password: String,
    #[validate(custom(function = "validate_password_policy"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ForgotPasswordDto {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordDto {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(custom(function = "validate_password_policy"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct VerifyEmailQuery {
    pub token: String,
}
