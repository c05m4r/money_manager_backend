// Copyright (C) 2026 Marcos Gabriel Miller
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chrono::Utc;
use rand_core::OsRng;
use sea_orm_migration::sea_orm::{
    ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr, Statement,
};
use seed_defaults::{
    seed_default_data_for_user, DEFAULT_ADMIN_EMAIL, DEFAULT_ADMIN_PASSWORD,
    DEFAULT_ADMIN_USERNAME, DEFAULT_AUDITOR_EMAIL, DEFAULT_AUDITOR_PASSWORD,
    DEFAULT_AUDITOR_USERNAME, DEFAULT_CURRENCIES, DEFAULT_MANAGER_EMAIL,
    DEFAULT_MANAGER_PASSWORD, DEFAULT_MANAGER_USERNAME, DEFAULT_TRANSACTION_TYPES,
    DEFAULT_USER_EMAIL, DEFAULT_USER_PASSWORD, DEFAULT_USER_USERNAME,
};
use uuid::Uuid;

pub async fn run(db: &DatabaseConnection) -> Result<(), DbErr> {
    let now = Utc::now().to_rfc3339();

    let admin_uuid = Uuid::now_v7().to_string();
    let manager_uuid = Uuid::now_v7().to_string();
    let auditor_uuid = Uuid::now_v7().to_string();
    let default_seed_uuid = Uuid::now_v7().to_string();

    ensure_user(
        db,
        &admin_uuid,
        DEFAULT_ADMIN_USERNAME,
        DEFAULT_ADMIN_EMAIL,
        DEFAULT_ADMIN_PASSWORD,
        "admin",
        &now,
    )
    .await?;
    ensure_user(
        db,
        &manager_uuid,
        DEFAULT_MANAGER_USERNAME,
        DEFAULT_MANAGER_EMAIL,
        DEFAULT_MANAGER_PASSWORD,
        "manager",
        &now,
    )
    .await?;
    ensure_user(
        db,
        &auditor_uuid,
        DEFAULT_AUDITOR_USERNAME,
        DEFAULT_AUDITOR_EMAIL,
        DEFAULT_AUDITOR_PASSWORD,
        "auditor",
        &now,
    )
    .await?;
    ensure_user(
        db,
        &default_seed_uuid,
        DEFAULT_USER_USERNAME,
        DEFAULT_USER_EMAIL,
        DEFAULT_USER_PASSWORD,
        "user",
        &now,
    )
    .await?;

    let default_uuid = user_uuid_by_email(db, DEFAULT_USER_EMAIL)
        .await?
        .ok_or_else(|| DbErr::Custom("default user not found after seeding".to_string()))?;

    for &(code, name, symbol) in DEFAULT_CURRENCIES {
        upsert_currency(db, code, name, symbol, &now).await?;
    }

    seed_default_data_for_user(db, default_uuid).await?;

    for &(name, code) in DEFAULT_TRANSACTION_TYPES {
        ensure_transaction_type(db, name, code, &now).await?;
    }

    Ok(())
}

async fn ensure_user(
    db: &DatabaseConnection,
    uuid: &str,
    username: &str,
    email: &str,
    password: &str,
    role: &str,
    now: &str,
) -> Result<(), DbErr> {
    if exists(
        db,
        format!(
            "SELECT 1 FROM users WHERE email = '{}' AND deleted_at IS NULL LIMIT 1",
            esc(email)
        ),
    )
    .await?
    {
        return Ok(());
    }

    let password_hash = hash_password(password)?;

    execute(
        db,
        format!(
            "INSERT INTO users (uuid, username, email, password_hash, role, description, is_active, email_verified_at, last_login_at, created_at, updated_at, deleted_at) \
             VALUES ('{}', '{}', '{}', '{}', '{}', NULL, TRUE, NULL, NULL, '{}', '{}', NULL)",
            esc(uuid),
            esc(username),
            esc(email),
            esc(&password_hash),
            esc(role),
            esc(now),
            esc(now)
        ),
    )
    .await
}

async fn upsert_currency(
    db: &DatabaseConnection,
    code: &str,
    name: &str,
    symbol: &str,
    now: &str,
) -> Result<(), DbErr> {
    execute(
        db,
        format!(
            "INSERT INTO currencies (code, name, symbol, created_at, updated_at, deleted_at) \
             VALUES ('{}', '{}', '{}', '{}', '{}', NULL) \
             ON CONFLICT (code) DO NOTHING",
            esc(code),
            esc(name),
            esc(symbol),
            esc(now),
            esc(now)
        ),
    )
    .await
}

async fn ensure_transaction_type(
    db: &DatabaseConnection,
    name: &str,
    code: &str,
    now: &str,
) -> Result<(), DbErr> {
    if exists(
        db,
        format!(
            "SELECT 1 FROM transaction_types WHERE code = '{}' AND deleted_at IS NULL LIMIT 1",
            esc(code)
        ),
    )
    .await?
    {
        return Ok(());
    }

    let tx_type_uuid = Uuid::now_v7().to_string();
    execute(
        db,
        format!(
            "INSERT INTO transaction_types (uuid, name, code, created_at, updated_at, deleted_at) \
             VALUES ('{}', '{}', '{}', '{}', '{}', NULL)",
            esc(&tx_type_uuid),
            esc(name),
            esc(code),
            esc(now),
            esc(now)
        ),
    )
    .await
}

async fn user_uuid_by_email(db: &DatabaseConnection, email: &str) -> Result<Option<Uuid>, DbErr> {
    let backend = db.get_database_backend();
    let row = db
        .query_one(Statement::from_string(
            backend,
            format!(
                "SELECT uuid::text AS uuid FROM users WHERE email = '{}' AND deleted_at IS NULL LIMIT 1",
                esc(email)
            ),
        ))
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let uuid_str: String = row.try_get("", "uuid")?;
    let uuid = Uuid::parse_str(&uuid_str)
        .map_err(|e| DbErr::Custom(format!("invalid uuid value in users.uuid: {e}")))?;

    Ok(Some(uuid))
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

fn hash_password(password: &str) -> Result<String, DbErr> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| DbErr::Custom(format!("failed to hash password: {e}")))
}

fn esc(value: &str) -> String {
    value.replace('\'', "''")
}
