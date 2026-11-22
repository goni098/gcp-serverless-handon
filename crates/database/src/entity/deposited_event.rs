use sea_orm::entity::prelude::*;
use serde::Serialize;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "deposited_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ix: i32,

    #[sea_orm(
        primary_key,
        auto_increment = false,
        column_type = "String(StringLen::N(88))"
    )]
    pub sig: String,

    #[sea_orm(column_type = "String(StringLen::N(44))")]
    pub mint: String,

    #[sea_orm(column_type = "String(StringLen::N(44))")]
    pub depositor: String,

    pub amount: i64,

    pub created_at: DateTimeWithTimeZone,
}

impl ActiveModelBehavior for ActiveModel {}
