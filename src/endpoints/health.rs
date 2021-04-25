use serde::Serialize;
use std::convert::Infallible;
use warp::Filter;

#[derive(Serialize)]
enum ServiceStatus {
    Ok,
    // TODO: Put other statuses in
}

#[derive(Serialize)]
struct Status {
    service_status: ServiceStatus,
}

pub async fn health_check_handler() -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::reply::json(&Status {
        service_status: ServiceStatus::Ok,
    }));
}

pub fn health_check_filter(
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("health-check").and_then(health_check_handler)
}
