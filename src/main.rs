use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::Filter;

const LABEL_KEY: &str = "actions-ephemeral-runner";
const LABEL_VALUE: &str = "true";
const RUNTIME_CLASS: &str = "edera";

#[tokio::main]
async fn main() {
    println!("starting server");
    let mutate = warp::post()
        .and(warp::path("mutate"))
        .and(warp::body::json())
        .map(|admission_review: AdmissionReview| {
            let response = handle_mutation(admission_review);
            println!("mutating {:?}", response);
            warp::reply::json(&response)
        });

    println!("listening on 8443");
    warp::serve(mutate)
        .tls()
        .cert_path("/certs/tls.crt")
        .key_path("/certs/tls.key")
        .run(([0, 0, 0, 0], 8443))
        .await;
}

fn handle_mutation(review: AdmissionReview) -> AdmissionReviewResponse {
    let request = review.request.unwrap();
    let pod = request.object;
    let metadata = pod.metadata.unwrap_or_default();
    let labels = metadata.clone().labels.unwrap_or_default();

    println!("Received request for {:?}", metadata.clone());

    // Check if the pod matches the label
    if let Some(value) = labels.get(LABEL_KEY) {
        if value.to_lowercase() == LABEL_VALUE {
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
            let patch_base64 = base64::encode(serde_json::to_string(&patch).unwrap());

            return AdmissionReviewResponse {
                api_version: "admission.k8s.io/v1".to_string(),
                kind: "AdmissionReview".to_string(),
                response: Some(Response {
                    uid: request.uid,
                    allowed: true,
                    patch_type: Some("JSONPatch".to_string()),
                    patch: Some(patch_base64),
                }),
            };
        }
    }

    // If no label matches, allow the pod without changes
    AdmissionReviewResponse {
        api_version: "admission.k8s.io/v1".to_string(),
        kind: "AdmissionReview".to_string(),
        response: Some(Response {
            uid: request.uid,
            allowed: true,
            patch_type: None,
            patch: None,
        }),
    }
}

#[derive(Deserialize, Debug)]
struct AdmissionReview {
    request: Option<AdmissionRequest>,
}

#[derive(Deserialize, Debug)]
struct AdmissionRequest {
    uid: String,
    object: Pod,
}

#[derive(Deserialize, Debug)]
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
