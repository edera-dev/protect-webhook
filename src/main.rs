use anyhow::Result;
use base64::prelude::*;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use warp::Filter;

const RUNTIME_CLASS: &str = "edera";

#[derive(Deserialize, Debug, Clone)]
struct AdmissionReview {
    request: Option<AdmissionRequest>,
}

#[derive(Deserialize, Debug, Clone)]
struct AdmissionRequest {
    uid: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AdmissionReviewResponse {
    api_version: String,
    kind: String,
    response: Option<Response>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Response {
    uid: String,
    allowed: bool,
    patch_type: Option<String>,
    patch: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

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

fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    mutate().or(livez()).or(healthz())
}

fn healthz() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("healthz")).map(|| {
        debug!("GET /healthz");
        warp::reply()
    })
}

fn livez() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path("livez")).map(|| {
        debug!("GET /livez");
        warp::reply()
    })
}

fn mutate() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("mutate"))
        .and(warp::body::json())
        .map(|admission_review: AdmissionReview| {
            let response = mutate_internal(admission_review).unwrap();
            info!("mutating {:?}", response);
            warp::reply::json(&response)
        })
}

fn mutate_internal(review: AdmissionReview) -> Result<AdmissionReviewResponse> {
    let request = review.request.clone().unwrap();
    let patch = json!([{
        "op": "add",
        "path": "/spec/runtimeClassName",
        "value": RUNTIME_CLASS
    }]);
    let patch_base64 = BASE64_STANDARD.encode(serde_json::to_string(&patch).unwrap());

    Ok(AdmissionReviewResponse {
        api_version: "admission.k8s.io/v1".to_string(),
        kind: "AdmissionReview".to_string(),
        response: Some(Response {
            uid: request.uid,
            allowed: true,
            patch_type: Some("JSONPatch".to_string()),
            patch: Some(patch_base64),
        }),
    })
}
