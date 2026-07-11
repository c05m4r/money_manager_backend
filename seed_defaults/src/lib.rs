// Copyright (C) 2026 Marcos Gabriel Miller
use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, Statement};
use uuid::Uuid;

pub const DEFAULT_ADMIN_USERNAME: &str = "admin";
pub const DEFAULT_ADMIN_EMAIL: &str = "admin@moneymanager.com";
pub const DEFAULT_ADMIN_PASSWORD: &str = "ContraseniaSegura2026!";

pub const DEFAULT_MANAGER_USERNAME: &str = "manager";
pub const DEFAULT_MANAGER_EMAIL: &str = "manager@moneymanager.com";
pub const DEFAULT_MANAGER_PASSWORD: &str = "ContraseniaSegura2026!";

pub const DEFAULT_AUDITOR_USERNAME: &str = "auditor";
pub const DEFAULT_AUDITOR_EMAIL: &str = "auditor@moneymanager.com";
pub const DEFAULT_AUDITOR_PASSWORD: &str = "ContraseniaSegura2026!";

pub const DEFAULT_USER_USERNAME: &str = "default";
pub const DEFAULT_USER_EMAIL: &str = "default@moneymanager.com";
pub const DEFAULT_USER_PASSWORD: &str = "ContraseniaSegura2026!";

pub const DEFAULT_ACCOUNT_NAME: &str = "default";
pub const DEFAULT_ACCOUNT_CURRENCY_CODE: &str = "ARS";

pub const DEFAULT_CURRENCIES: &[(&str, &str, &str)] = &[
    ("ARS", "Argentine Peso", "$"),
    ("USD", "US Dollar", "$"),
    ("EUR", "Euro", "€"),
    ("BRL", "Brazilian Real", "R$"),
    ("CLP", "Chilean Peso", "$"),
    ("COP", "Colombian Peso", "$"),
    ("MXN", "Mexican Peso", "$"),
    ("PEN", "Peruvian Sol", "S/"),
    ("UYU", "Uruguayan Peso", "$"),
    ("PYG", "Paraguayan Guarani", "₲"),
    ("BOB", "Bolivian Boliviano", "Bs"),
    ("GBP", "Pound Sterling", "£"),
    ("JPY", "Japanese Yen", "¥"),
    ("CNY", "Chinese Yuan", "¥"),
    ("CHF", "Swiss Franc", "CHF"),
    ("CAD", "Canadian Dollar", "C$"),
    ("AUD", "Australian Dollar", "A$"),
    ("NZD", "New Zealand Dollar", "NZ$"),
];

pub const DEFAULT_TRANSACTION_TYPES: &[(&str, &str)] = &[("income", "IN"), ("expenses", "OUT")];

pub const DEFAULT_CATEGORIES: &[&str] = &[
    "Debt",
    "Miscellaneous",
    "Services",
    "Clothes",
    "Groceries",
    "Transportation",
    "Coffee",
    "Workout",
    "Gifts",
    "Family",
    "Health",
    "Leisure",
    "Education",
    "Other",
    "Home",
    "Work",
    "Cashback",
    "Gift",
    "Interest",
    "Paycheck",
];

pub async fn seed_default_data_for_user(
    db: &DatabaseConnection,
    user_uuid: Uuid,
) -> Result<(), DbErr> {
    let now = Utc::now().to_rfc3339();

    ensure_default_account(db, &user_uuid, &now).await?;

    for &category in DEFAULT_CATEGORIES {
        ensure_category(db, &user_uuid, category, &now).await?;
    }

    Ok(())
}

async fn ensure_default_account(
    db: &DatabaseConnection,
    user_uuid: &Uuid,
    now: &str,
) -> Result<(), DbErr> {
    if exists(
        db,
        format!(
            "SELECT 1 FROM accounts WHERE user_uuid = '{}' AND currency_code = '{}' AND name = '{}' AND deleted_at IS NULL LIMIT 1",
            esc(&user_uuid.to_string()),
            esc(DEFAULT_ACCOUNT_CURRENCY_CODE),
            esc(DEFAULT_ACCOUNT_NAME)
        ),
    )
    .await?
    {
        return Ok(());
    }

    let account_uuid = Uuid::now_v7().to_string();
    execute(
        db,
        format!(
             "INSERT INTO accounts (uuid, user_uuid, currency_code, name, is_default, created_at, updated_at, deleted_at) \
             VALUES ('{}', '{}', '{}', '{}', true, '{}', '{}', NULL)",
            esc(&account_uuid),
            esc(&user_uuid.to_string()),
            esc(DEFAULT_ACCOUNT_CURRENCY_CODE),
            esc(DEFAULT_ACCOUNT_NAME),
            esc(now),
            esc(now)
        ),
    )
    .await
}

async fn ensure_category(
    db: &DatabaseConnection,
    user_uuid: &Uuid,
    name: &str,
    now: &str,
) -> Result<(), DbErr> {
    if exists(
        db,
        format!(
            "SELECT 1 FROM categories WHERE user_uuid = '{}' AND name = '{}' AND deleted_at IS NULL LIMIT 1",
            esc(&user_uuid.to_string()),
            esc(name)
        ),
    )
    .await?
    {
        return Ok(());
    }

    let category_uuid = Uuid::now_v7().to_string();
    execute(
        db,
        format!(
            "INSERT INTO categories (uuid, user_uuid, name, created_at, updated_at, deleted_at) \
             VALUES ('{}', '{}', '{}', '{}', '{}', NULL)",
            esc(&category_uuid),
            esc(&user_uuid.to_string()),
            esc(name),
            esc(now),
            esc(now)
        ),
    )
    .await
}

async fn exists(db: &DatabaseConnection, sql: String) -> Result<bool, DbErr> {
    let backend = db.get_database_backend();
    let row = db.query_one(Statement::from_string(backend, sql)).await?;
    Ok(row.is_some())
}

async fn execute(db: &DatabaseConnection, sql: String) -> Result<(), DbErr> {
    let backend: DatabaseBackend = db.get_database_backend();
    db.execute(Statement::from_string(backend, sql)).await?;
    Ok(())
}

fn esc(value: &str) -> String {
    value.replace('\'', "''")
}
