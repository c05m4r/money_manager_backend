// Copyright (C) 2026 Marcos Gabriel Miller
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::{
    handlers,
    models::{
        accounts::{AccountListQuery, AccountResponse, CreateAccountDto, UpdateAccountDto},
        audit_logs::{AuditLogListQuery, AuditLogResponse},
        categories::{CategoryListQuery, CategoryResponse, CreateCategoryDto, UpdateCategoryDto},
        currencies::{CreateCurrencyDto, CurrencyListQuery, CurrencyResponse, UpdateCurrencyDto},
        pagination::{Links, Meta},
        transaction_types::{
            CreateTransactionTypeDto, TransactionTypeListQuery, TransactionTypeResponse,
            UpdateTransactionTypeDto,
        },
        transactions::{
            CreateTransactionDto, TransactionListQuery, TransactionResponse, UpdateTransactionDto,
        },
        users::{
            ChangePasswordDto, CreateUserDto, ForgotPasswordDto, LoginDto, LoginResponse,
            ResetPasswordDto, UpdateUserDto, UserListQuery, UserResponse, VerifyEmailQuery,
        },
    },
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Money Manager API",
        version = "0.1.0",
        description = "REST API for the Money Manager backend"
    ),
    paths(
        handlers::system::health,
        handlers::auth::forgot_password,
        handlers::auth::reset_password,
        handlers::auth::login,
        handlers::auth::verify_email,
        handlers::auth::resend_verification_email,
        handlers::auth::change_password,
        handlers::auth::logout,
        handlers::users::list,
        handlers::users::get_one,
        handlers::users::create,
        handlers::users::update,
        handlers::users::delete,
        handlers::currencies::list,
        handlers::currencies::get_one,
        handlers::currencies::create,
        handlers::currencies::update,
        handlers::currencies::delete,
        handlers::accounts::list,
        handlers::accounts::get_one,
        handlers::accounts::create,
        handlers::accounts::update,
        handlers::accounts::delete,
        handlers::categories::list,
        handlers::categories::get_one,
        handlers::categories::create,
        handlers::categories::update,
        handlers::categories::delete,
        handlers::transaction_types::list,
        handlers::transaction_types::get_one,
        handlers::transaction_types::create,
        handlers::transaction_types::update,
        handlers::transaction_types::delete,
        handlers::transactions::list,
        handlers::transactions::get_one,
        handlers::transactions::create,
        handlers::transactions::update,
        handlers::transactions::delete,
        handlers::audit_logs::list,
        handlers::audit_logs::get_one,
    ),
    components(schemas(
        UserResponse,
        CreateUserDto,
        UpdateUserDto,
        UserListQuery,
        LoginDto,
        LoginResponse,
        ChangePasswordDto,
        ForgotPasswordDto,
        ResetPasswordDto,
        VerifyEmailQuery,
        CurrencyResponse,
        CreateCurrencyDto,
        UpdateCurrencyDto,
        CurrencyListQuery,
        AccountResponse,
        CreateAccountDto,
        UpdateAccountDto,
        AccountListQuery,
        CategoryResponse,
        CreateCategoryDto,
        UpdateCategoryDto,
        CategoryListQuery,
        TransactionTypeResponse,
        CreateTransactionTypeDto,
        UpdateTransactionTypeDto,
        TransactionTypeListQuery,
        TransactionResponse,
        CreateTransactionDto,
        UpdateTransactionDto,
        TransactionListQuery,
        AuditLogResponse,
        AuditLogListQuery,
        Meta,
        Links,
    )),
    tags(
        (name = "System", description = "Health endpoint"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Users", description = "User management"),
        (name = "Currencies", description = "Currency management"),
        (name = "Accounts", description = "Account management"),
        (name = "Categories", description = "Category management"),
        (name = "Transaction Types", description = "Transaction type management"),
        (name = "Transactions", description = "Transaction management"),
        (name = "Audit Logs", description = "Audit log access"),
    ),
    modifiers(&SecurityAddon),
    security(("bearer_auth" = []))
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
