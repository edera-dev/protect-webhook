use anyhow::{anyhow, Result};
use log::info;
use std::{env, fs};
use warp::Filter;

mod healthz;
mod livez;
mod mutate;

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    mutate::handler()
        .or(livez::handler())
        .or(healthz::handler())
}

fn set_certs_dir() -> Result<String> {
    let certs_dir = env::var("WEBHOOK_CERTS_DIR").unwrap_or("/certs".to_string());

    let meta = match fs::metadata(&certs_dir) {
        Err(e) => return Err(anyhow!("Error reading metadata for {}: {}", certs_dir, e)),
        Ok(meta) => meta,
    };

    if !meta.is_dir() {
        return Err(anyhow!("{} is not a directory", certs_dir));
    }

    Ok(certs_dir)
}

fn set_crt_path(certs_dir: &String) -> Result<String> {
    let crt_file = env::var("WEBHOOK_CRT_FILE").unwrap_or("tls.crt".to_string());
    let crt_path = format!("{}/{}", certs_dir, &crt_file);

    let meta = match fs::metadata(&crt_path) {
        Err(e) => return Err(anyhow!("Error reading metadata for {}: {}", crt_file, e)),
        Ok(meta) => meta,
    };

    if !meta.is_file() {
        return Err(anyhow!("{} is not a file", crt_path));
    }

    Ok(crt_path)
}

fn set_key_path(certs_dir: &String) -> Result<String> {
    let key_file = env::var("WEBHOOK_KEY_FILE").unwrap_or("tls.key".to_string());
    let key_path = format!("{}/{}", certs_dir, &key_file);

    let meta = match fs::metadata(&key_path) {
        Err(e) => return Err(anyhow!("Error reading metadata for {}: {}", key_file, e)),
        Ok(meta) => meta,
    };

    if !meta.is_file() {
        return Err(anyhow!("{} is not a file", key_path));
    }

    Ok(key_path)
}

pub async fn start() -> Result<()> {
    let certs_dir = set_certs_dir()?;
    let crt_path = set_crt_path(&certs_dir)?;
    let key_path = set_key_path(&certs_dir)?;
    info!("configured certs directory to: {}", certs_dir);

    // TODO: Make healthz and livez listen on http rather than https if they need to do more
    let routes = routes();

    info!("listening on 8443");
    warp::serve(routes)
        .tls()
        .cert_path(crt_path)
        .key_path(key_path)
        .run(([0, 0, 0, 0], 8443))
        .await;

    Ok(())
}
