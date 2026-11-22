pub use sea_orm_migration::prelude::*;

mod m20261121_073644_init;
mod m20261121_075343_drop_col_nnn;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261121_073644_init::Migration),
            Box::new(m20261121_075343_drop_col_nnn::Migration),
        ]
    }
}
