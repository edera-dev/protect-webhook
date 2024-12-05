use anyhow::Result;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use warp::Filter;

const RUNTIME_CLASS: &str = "edera";

#[derive(Deserialize, Debug, Clone)]
struct AdmissionReview {
    request: Option<AdmissionRequest>,
}

#[derive(Deserialize, Debug, Clone)]
struct AdmissionRequest {
    uid: String,
    object: Pod,
}

#[derive(Deserialize, Debug, Clone)]
struct Pod {
    metadata: Option<Metadata>,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct Metadata {
    labels: Option<std::collections::HashMap<String, String>>,
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
    println!("starting server");
    let mutate = warp::post()
        .and(warp::path("mutate"))
        .and(warp::body::json())
        .map(|admission_review: AdmissionReview| {
            let response = mutate(admission_review).unwrap();
            println!("mutating {:#?}", response);
            warp::reply::json(&response)
        });

    println!("listening on 8443");
    warp::serve(mutate)
        .tls()
        .cert_path("/certs/tls.crt")
        .key_path("/certs/tls.key")
        .run(([0, 0, 0, 0], 8443))
        .await;

    Ok(())
}

fn should_mutate(review: AdmissionReview) -> Result<bool> {
    let request = review.request.unwrap();
    let pod = request.object;
    let metadata = pod.metadata.unwrap_or_default();
    let labels = metadata.clone().labels.unwrap_or_default();

    println!("Received request for {:#?}", metadata.clone());

    let selectors = HashMap::from([
        ("actions-ephemeral-runner", "true"),
        ("actions.github.com/scale-set-name", "arc-runner-set-edera"),
    ]);

    let mut matched = 0;
    for (key, value) in selectors.clone() {
        let Some(label_value) = labels.get(key) else {
            continue;
        };

        if label_value.to_lowercase() == value {
            matched += 1;
        }
    }
    Ok(matched == selectors.len())
}

fn mutate(review: AdmissionReview) -> Result<AdmissionReviewResponse> {
    let request = review.request.clone().unwrap();
    if should_mutate(review)? {
        // Create a JSONPatch to add runtimeClassName
        let patch = json!([{
            "op": "add",
            "path": "/spec/runtimeClassName",
            "value": RUNTIME_CLASS
        },{
            "op": "add",
            "path": "/metadata/annotations/dev.edera~1kernel_verbose",
            "value": "true"
        }]);
        let patch_base64 = BASE64_STANDARD.encode(serde_json::to_string(&patch).unwrap());

        return Ok(AdmissionReviewResponse {
            api_version: "admission.k8s.io/v1".to_string(),
            kind: "AdmissionReview".to_string(),
            response: Some(Response {
                uid: request.uid,
                allowed: true,
                patch_type: Some("JSONPatch".to_string()),
                patch: Some(patch_base64),
            }),
        });
    }

    // If no label matches, allow the pod without changes
    Ok(AdmissionReviewResponse {
        api_version: "admission.k8s.io/v1".to_string(),
        kind: "AdmissionReview".to_string(),
        response: Some(Response {
            uid: request.uid,
            allowed: true,
            patch_type: None,
            patch: None,
        }),
    })
}
