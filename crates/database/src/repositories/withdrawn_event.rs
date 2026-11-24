use app_error::{AppError, R};
use chrono::Utc;
use sea_orm::{ConnectionTrait, EntityTrait, Set};

use crate::entity::withdrawn_event::{ActiveModel, Entity};

pub async fn save<C: ConnectionTrait>(
    db: &C,
    sig: &str,
    ix: i32,
    mint: &str,
    authority: &str,
    amount: u64,
) -> R<()> {
    let amount = i64::try_from(amount).map_err(|_| {
        AppError::cust(format!(
            "withdrawal amount {amount} exceeds PostgreSQL BIGINT"
        ))
    })?;

    Entity::insert(ActiveModel {
        ix: Set(ix),
        sig: Set(sig.to_owned()),
        mint: Set(mint.to_owned()),
        authority: Set(authority.to_owned()),
        amount: Set(amount),
        created_at: Set(Utc::now().into()),
    })
    .exec(db)
    .await?;

    Ok(())
}
