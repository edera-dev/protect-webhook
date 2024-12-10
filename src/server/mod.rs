use anyhow::Result;
use log::info;
use std::env;
use warp::Filter;

mod healthz;
mod livez;
mod mutate;

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    mutate::handler()
        .or(livez::handler())
        .or(healthz::handler())
}

pub async fn start() -> Result<()> {
    let certs_dir = env::var("WEBHOOK_CERTS_DIR").unwrap_or("/certs".to_string());
    let crt_file = env::var("WEBHOOK_CRT_FILE").unwrap_or("tls.crt".to_string());
    let key_file = env::var("WEBHOOK_KEY_FILE").unwrap_or("tls.key".to_string());
    info!("configured certs directory to: {}", certs_dir);

    // TODO: Make healthz and livez listen on http rather than https if they need to do more
    let routes = routes();

    info!("listening on 8443");
    warp::serve(routes)
        .tls()
        .cert_path(format!("{}/{}", certs_dir, crt_file))
        .key_path(format!("{}/{}", certs_dir, key_file))
        .run(([0, 0, 0, 0], 8443))
        .await;

    Ok(())
}
