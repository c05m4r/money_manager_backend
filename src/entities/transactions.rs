// Copyright (C) 2026 Marcos Gabriel Miller
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub amount: Decimal,
    pub description: Option<String>,
    pub transaction_date: DateTimeUtc,
    pub type_uuid: Uuid,
    pub category_uuid: Option<Uuid>,
    pub account_uuid: Option<Uuid>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction_types::Entity",
        from = "Column::TypeUuid",
        to = "super::transaction_types::Column::Uuid"
    )]
    TransactionType,
    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::CategoryUuid",
        to = "super::categories::Column::Uuid"
    )]
    Category,
    #[sea_orm(
        belongs_to = "super::accounts::Entity",
        from = "Column::AccountUuid",
        to = "super::accounts::Column::Uuid"
    )]
    Account,
}

impl Related<super::transaction_types::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TransactionType.def()
    }
}

impl Related<super::categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl Related<super::accounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
