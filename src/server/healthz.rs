use log::debug;
use warp::Filter;

pub fn handler() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("healthz")).map(|| {
        debug!("GET /healthz");
        warp::reply()
    })
}
