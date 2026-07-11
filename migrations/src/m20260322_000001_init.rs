// Copyright (C) 2026 Marcos Gabriel Miller
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).string().not_null())
                    .col(ColumnDef::new(Users::Role).string().not_null().default("user"))
                    .col(ColumnDef::new(Users::Description).string().null())
                    .col(ColumnDef::new(Users::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(Users::EmailVerifiedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Users::LastLoginAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Users::DeletedAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Currencies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Currencies::Code).string().not_null().primary_key())
                    .col(ColumnDef::new(Currencies::Name).string().not_null())
                    .col(ColumnDef::new(Currencies::Symbol).string().not_null())
                    .col(ColumnDef::new(Currencies::Description).string().null())
                    .col(ColumnDef::new(Currencies::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Currencies::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Currencies::DeletedAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Accounts::UserUuid).uuid().not_null())
                    .col(ColumnDef::new(Accounts::CurrencyCode).string().not_null())
                    .col(ColumnDef::new(Accounts::Name).string().not_null())
                    .col(ColumnDef::new(Accounts::Description).string().null())
                    .col(ColumnDef::new(Accounts::IsDefault).boolean().not_null().default(false))
                    .col(ColumnDef::new(Accounts::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Accounts::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Accounts::DeletedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_user_uuid")
                            .from(Accounts::Table, Accounts::UserUuid)
                            .to(Users::Table, Users::Uuid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_accounts_currency_code")
                            .from(Accounts::Table, Accounts::CurrencyCode)
                            .to(Currencies::Table, Currencies::Code),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Categories::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Categories::UserUuid).uuid().not_null())
                    .col(ColumnDef::new(Categories::Name).string().not_null())
                    .col(ColumnDef::new(Categories::Description).string().null())
                    .col(ColumnDef::new(Categories::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Categories::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Categories::DeletedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_user_uuid")
                            .from(Categories::Table, Categories::UserUuid)
                            .to(Users::Table, Users::Uuid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TransactionTypes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransactionTypes::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TransactionTypes::Name).string().not_null())
                    .col(
                        ColumnDef::new(TransactionTypes::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(TransactionTypes::Description).string().null())
                    .col(ColumnDef::new(TransactionTypes::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TransactionTypes::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TransactionTypes::DeletedAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transactions::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transactions::Amount).decimal().not_null())
                    .col(ColumnDef::new(Transactions::Description).string().null())
                    .col(ColumnDef::new(Transactions::TransactionDate).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Transactions::TypeUuid).uuid().not_null())
                    .col(ColumnDef::new(Transactions::CategoryUuid).uuid().null())
                    .col(ColumnDef::new(Transactions::AccountUuid).uuid().null())
                    .col(ColumnDef::new(Transactions::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Transactions::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Transactions::DeletedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_type_uuid")
                            .from(Transactions::Table, Transactions::TypeUuid)
                            .to(TransactionTypes::Table, TransactionTypes::Uuid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_category_uuid")
                            .from(Transactions::Table, Transactions::CategoryUuid)
                            .to(Categories::Table, Categories::Uuid),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_transactions_account_uuid")
                            .from(Transactions::Table, Transactions::AccountUuid)
                            .to(Accounts::Table, Accounts::Uuid),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuditLogs::Uuid).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuditLogs::UserUuid).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::TableName).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Description).string().null())
                    .col(ColumnDef::new(AuditLogs::RecordUuid).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::OldValues).json_binary().null())
                    .col(ColumnDef::new(AuditLogs::NewValues).json_binary().null())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string().not_null())
                    .col(ColumnDef::new(AuditLogs::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user_uuid")
                            .from(AuditLogs::Table, AuditLogs::UserUuid)
                            .to(Users::Table, Users::Uuid),
                    )
                    .to_owned(),
            )
            .await?;

        // Partial unique indexes to allow soft-deleted rows to reuse email/username
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_key;

                DROP INDEX IF EXISTS users_email_unique_active;
                CREATE UNIQUE INDEX users_email_unique_active
                    ON users (email)
                    WHERE deleted_at IS NULL;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Transactions::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(TransactionTypes::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Accounts::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Currencies::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Uuid,
    Username,
    Email,
    PasswordHash,
    Role,
    Description,
    IsActive,
    EmailVerifiedAt,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Currencies {
    Table,
    Code,
    Name,
    Symbol,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Accounts {
    Table,
    Uuid,
    UserUuid,
    CurrencyCode,
    Name,
    Description,
    IsDefault,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Categories {
    Table,
    Uuid,
    UserUuid,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum TransactionTypes {
    Table,
    Uuid,
    Name,
    Code,
    Description,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum Transactions {
    Table,
    Uuid,
    Amount,
    Description,
    TransactionDate,
    TypeUuid,
    CategoryUuid,
    AccountUuid,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum AuditLogs {
    Table,
    Uuid,
    UserUuid,
    Action,
    TableName,
    Description,
    RecordUuid,
    OldValues,
    NewValues,
    IpAddress,
    CreatedAt,
}
