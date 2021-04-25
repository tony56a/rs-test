mod hello;
mod health;
pub mod errors;

use warp::Filter;

pub fn filters() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    hello::hello_filter().or(health::health_check_filter())
}