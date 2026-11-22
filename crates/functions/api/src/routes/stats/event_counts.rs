use axum::Json;
use database::repositories;
use database::repositories::stats::EventCounts;
use http::exception::Rh;
use http::extractors::Pool;

pub async fn handler(Pool(db): Pool) -> Rh<Json<EventCounts>> {
    let counts = repositories::stats::count_events(&db).await?;
    Ok(Json(counts))
}
