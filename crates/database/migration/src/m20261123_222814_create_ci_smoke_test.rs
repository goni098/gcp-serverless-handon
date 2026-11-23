use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

const CI_SMOKE_TEST: &str = "ci_smoke_test";

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261123_222814_create_ci_smoke_test"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CI_SMOKE_TEST)
                    .col(pk_auto("id"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CI_SMOKE_TEST).to_owned())
            .await
    }
}
