// Copyright (C) 2026 Marcos Gabriel Miller
use sea_orm_migration::sea_orm::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_filename("env/.env")
        .or_else(|_| dotenvy::dotenv())
        .ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL no está configurada en el entorno")?;

    let db = Database::connect(&database_url).await?;
    migrations::seed(&db).await?;

    println!("Seeders ejecutados correctamente");
    Ok(())
}
