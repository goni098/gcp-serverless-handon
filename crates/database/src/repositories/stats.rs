use app_error::R;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};
use serde::Serialize;

use crate::entity::{deposited_event, initialized_vault_event, withdrawn_event};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EventCounts {
    pub deposited: u64,
    pub initialized_vault: u64,
    pub withdrawn: u64,
}

pub async fn count_events(db: &DatabaseConnection) -> R<EventCounts> {
    let deposited = deposited_event::Entity::find().count(db).await?;
    let initialized_vault = initialized_vault_event::Entity::find().count(db).await?;
    let withdrawn = withdrawn_event::Entity::find().count(db).await?;

    Ok(EventCounts {
        deposited,
        initialized_vault,
        withdrawn,
    })
}
