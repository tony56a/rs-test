#[derive(Debug)]
pub struct InternalError;
impl warp::reject::Reject for InternalError {}