// Copyright (C) 2026 Marcos Gabriel Miller
use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn init_db(database_url: &str) -> DatabaseConnection {
    Database::connect(database_url)
        .await
        .expect("failed to connect to db")
}

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    migrations::up(db).await?;
    migrations::seed(db).await?;
    Ok(())
}
