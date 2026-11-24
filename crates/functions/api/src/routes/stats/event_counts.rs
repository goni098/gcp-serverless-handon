use axum::Json;
use database::repositories;
use database::repositories::stats::EventCounts;
use shared_function::exception::Rh;
use shared_function::extractors::Pool;

pub async fn handler(Pool(db): Pool) -> Rh<Json<EventCounts>> {
    let counts = repositories::stats::count_events(&db).await?;
    Ok(Json(counts))
}
