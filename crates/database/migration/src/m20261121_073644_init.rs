use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

// Preserve the spelling used by the existing entity and the original schema.
const INITIALIZED_VAULT_EVENTS: &str = "intitialized_vault_events";
const DEPOSITED_EVENTS: &str = "deposited_events";
const WITHDRAWN_EVENTS: &str = "withdrawn_events";

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261121_073644_init"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(INITIALIZED_VAULT_EVENTS)
                    .col(integer("ix"))
                    .col(string_len("sig", 88))
                    .col(string_len("mint", 44))
                    .col(string_len("authority", 44))
                    .col(timestamp_with_time_zone("created_at"))
                    .col(big_integer_null("nnn"))
                    .primary_key(Index::create().col("ix").col("sig"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DEPOSITED_EVENTS)
                    .col(integer("ix"))
                    .col(string_len("sig", 88))
                    .col(string_len("mint", 44))
                    .col(string_len("depositor", 44))
                    .col(big_integer("amount"))
                    .col(timestamp_with_time_zone("created_at"))
                    .primary_key(Index::create().col("ix").col("sig"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WITHDRAWN_EVENTS)
                    .col(integer("ix"))
                    .col(string_len("sig", 88))
                    .col(string_len("mint", 44))
                    .col(string_len("authority", 44))
                    .col(big_integer("amount"))
                    .col(timestamp_with_time_zone("created_at"))
                    .primary_key(Index::create().col("ix").col("sig"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [WITHDRAWN_EVENTS, DEPOSITED_EVENTS, INITIALIZED_VAULT_EVENTS] {
            manager
                .drop_table(Table::drop().table(table).to_owned())
                .await?;
        }

        Ok(())
    }
}
