// Copyright (C) 2026 Marcos Gabriel Miller
use seed_defaults::seed_default_data_for_user as seed_shared_default_data_for_user;
use uuid::Uuid;

use crate::errors::AppError;

/// Seeds default categories for a newly registered user.
/// Skips any category that the user already has (idempotent).
pub async fn seed_default_data_for_user(
    db: &sea_orm::DatabaseConnection,
    user_uuid: Uuid,
) -> Result<(), AppError> {
    seed_shared_default_data_for_user(db, user_uuid).await?;
    Ok(())
}
