// Copyright (C) 2026 Marcos Gabriel Miller
pub use sea_orm_migration::prelude::*;

mod m20260322_000001_init;
mod seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260322_000001_init::Migration),
        ]
    }
}

pub async fn up(
    db: &sea_orm_migration::sea_orm::DatabaseConnection,
) -> Result<(), sea_orm_migration::sea_orm::DbErr> {
    Migrator::up(db, None).await
}

pub async fn seed(
    db: &sea_orm_migration::sea_orm::DatabaseConnection,
) -> Result<(), sea_orm_migration::sea_orm::DbErr> {
    seeder::run(db).await
}
