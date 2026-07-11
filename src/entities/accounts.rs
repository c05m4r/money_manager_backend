// Copyright (C) 2026 Marcos Gabriel Miller
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub user_uuid: Uuid,
    pub currency_code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserUuid",
        to = "super::users::Column::Uuid"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::currencies::Entity",
        from = "Column::CurrencyCode",
        to = "super::currencies::Column::Code"
    )]
    Currency,
    #[sea_orm(has_many = "super::transactions::Entity")]
    SourceTransactions,
    #[sea_orm(has_many = "super::transactions::Entity")]
    TargetTransactions,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::currencies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Currency.def()
    }
}

impl Related<super::transactions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SourceTransactions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
