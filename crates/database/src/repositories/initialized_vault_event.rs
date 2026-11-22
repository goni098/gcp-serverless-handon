use chrono::Utc;
use error::R;
use sea_orm::{ConnectionTrait, EntityTrait, Set};

use crate::entity::initialized_vault_event::{ActiveModel, Entity};

pub async fn save<C: ConnectionTrait>(
    db: &C,
    sig: &str,
    ix: i32,
    mint: &str,
    authority: &str,
) -> R<()> {
    Entity::insert(ActiveModel {
        ix: Set(ix),
        sig: Set(sig.to_owned()),
        mint: Set(mint.to_owned()),
        authority: Set(authority.to_owned()),
        created_at: Set(Utc::now().into()),
    })
    .exec(db)
    .await?;

    Ok(())
}
