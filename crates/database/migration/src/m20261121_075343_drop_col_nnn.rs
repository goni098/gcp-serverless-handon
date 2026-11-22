use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

const INITIALIZED_VAULT_EVENTS: &str = "intitialized_vault_events";

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261121_075343_drop_col_nnn"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(INITIALIZED_VAULT_EVENTS)
                    .drop_column("nnn")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(INITIALIZED_VAULT_EVENTS)
                    .add_column(big_integer_null("nnn"))
                    .to_owned(),
            )
            .await
    }
}
