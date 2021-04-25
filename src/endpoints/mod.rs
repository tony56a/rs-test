pub mod errors;
mod health;
mod hello;

use warp::Filter;

pub fn filters() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    hello::hello_filter().or(health::health_check_filter())
}
