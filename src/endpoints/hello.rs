use warp::{http, Filter};
use std::convert::Infallible;

pub async fn hello_handler() -> Result<impl warp::Reply, Infallible> {
    return Ok(warp::reply::with_status("Hello World!", http::StatusCode::OK));
}

pub fn hello_filter() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    warp::path!("hello")
        .and_then(hello_handler)
}