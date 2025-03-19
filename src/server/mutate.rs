use anyhow::Result;
use base64::prelude::*;
use bytes::Bytes;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::Filter;

#[derive(Deserialize, Debug, Clone)]
struct AdmissionReview {
    request: Option<AdmissionRequest>,
}

#[derive(Deserialize, Debug, Clone)]
struct AdmissionRequest {
    uid: String,
    kind: Option<KindInfo>,
    object: K8sObject,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    namespace: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct K8sObject {
    metadata: Metadata,
    // include spec if needed in the future
    // spec: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Clone)]
struct Metadata {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "generateName")]
    generate_name: Option<String>,
    namespace: String,
}

#[derive(Deserialize, Debug, Clone)]
struct KindInfo {
    kind: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AdmissionReviewResponse {
    api_version: String,
    kind: String,
    response: Option<Response>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Response {
    uid: String,
    allowed: bool,
    patch_type: Option<String>,
    patch: Option<String>,
}

#[derive(Debug)]
struct JsonDeserializeError {
    message: String,
}

impl warp::reject::Reject for JsonDeserializeError {}

pub fn handler() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let base_path = warp::path!("mutate");

    base_path
        .and(warp::post())
        .and(warp::body::bytes())
        .and_then(log_and_deserialize)
        .and_then(mutate_internal)
}

async fn log_and_deserialize(body: Bytes) -> Result<AdmissionReview, warp::Rejection> {
    let raw_body = String::from_utf8_lossy(&body);
    debug!("Received raw body:\n{}", raw_body);
    serde_json::from_slice::<AdmissionReview>(&body).map_err(|err| {
        error!("Failed to deserialize AdmissionReview: {:?}", err);
        warp::reject::custom(JsonDeserializeError {
            message: err.to_string(),
        })
    })
}

async fn mutate_internal(review: AdmissionReview) -> Result<impl warp::Reply, warp::Rejection> {
    let Some(request) = review.request.clone() else {
        error!("failed to decode request");
        let error_response = json!({
            "error": "Invalid input",
            "code": 400
        });
        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    };

    // Get name, falling back to generateName if name is missing.
    let metadata = request.object.metadata;
    let name = metadata.name.unwrap_or_else(|| {
        metadata
            .generate_name
            .unwrap_or_else(|| "unknown".to_string())
    });
    let namespace = metadata.namespace;

    // Determine patch path based on object kind. Default to pod.
    let patch_path = if let Some(kind_info) = &request.kind {
        match kind_info.kind.as_str() {
            "Deployment" | "ReplicaSet" | "StatefulSet" | "DaemonSet" => {
                "/spec/template/spec/runtimeClassName"
            }
            _ => "/spec/runtimeClassName",
        }
    } else {
        "/spec/runtimeClassName"
    };

    let patch_template = format!(
        r#"[{{ "op": "add", "path": "{}", "value": "edera" }}]"#,
        patch_path
    );
    let patch = BASE64_STANDARD.encode(patch_template.as_bytes());

    let response = AdmissionReviewResponse {
        api_version: "admission.k8s.io/v1".to_string(),
        kind: "AdmissionReview".to_string(),
        response: Some(Response {
            uid: request.uid,
            allowed: true,
            patch_type: Some("JSONPatch".to_string()),
            patch: Some(patch),
        }),
    };

    info!("mutating {}/{}", namespace, name);
    debug!("payload {:?}", response);
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use warp::test::request;
    use warp::Reply;

    #[tokio::test]
    async fn test_mutate_pod() {
        let admission_review = AdmissionReview {
            request: Some(AdmissionRequest {
                uid: "test-uid".to_string(),
                kind: Some(KindInfo {
                    kind: "Pod".to_string(),
                }),
                object: K8sObject {
                    metadata: Metadata {
                        name: Some("test-name".to_string()),
                        generate_name: None,
                        namespace: "test-namespace".to_string(),
                    },
                },
                name: None,
                namespace: None,
            }),
        };

        let response = mutate_internal(admission_review).await.unwrap();
        let body = warp::hyper::body::to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let result: AdmissionReviewResponse = serde_json::from_slice(&body).unwrap();

        // Check the response structure
        assert_eq!(result.api_version, "admission.k8s.io/v1");
        assert_eq!(result.kind, "AdmissionReview");
        assert!(result.response.is_some());

        let response = result.response.unwrap();
        assert_eq!(response.uid, "test-uid");
        assert!(response.allowed);
        assert_eq!(response.patch_type, Some("JSONPatch".to_string()));

        // Decode and verify the patch for a Pod
        let patch_base64 = response.patch.unwrap();
        let patch_json: Value = serde_json::from_str(
            &String::from_utf8(BASE64_STANDARD.decode(patch_base64).unwrap()).unwrap(),
        )
        .unwrap();

        let expected_patch = json!([{
            "op": "add",
            "path": "/spec/runtimeClassName",
            "value": "edera",
        }]);

        assert_eq!(patch_json, expected_patch);
    }

    #[tokio::test]
    async fn test_mutate_replicaset() {
        let admission_review = AdmissionReview {
            request: Some(AdmissionRequest {
                uid: "replicaset-uid".to_string(),
                kind: Some(KindInfo {
                    kind: "ReplicaSet".to_string(),
                }),
                object: K8sObject {
                    metadata: Metadata {
                        name: Some("rs-name".to_string()),
                        generate_name: None,
                        namespace: "rs-namespace".to_string(),
                    },
                },
                name: None,
                namespace: None,
            }),
        };

        let response = mutate_internal(admission_review).await.unwrap();
        let body = warp::hyper::body::to_bytes(response.into_response().into_body())
            .await
            .unwrap();
        let result: AdmissionReviewResponse = serde_json::from_slice(&body).unwrap();

        // Check the response structure
        assert_eq!(result.api_version, "admission.k8s.io/v1");
        assert_eq!(result.kind, "AdmissionReview");
        assert!(result.response.is_some());

        let response = result.response.unwrap();
        assert_eq!(response.uid, "replicaset-uid");
        assert!(response.allowed);
        assert_eq!(response.patch_type, Some("JSONPatch".to_string()));

        // Decode and verify the patch for a ReplicaSet
        let patch_base64 = response.patch.unwrap();
        let patch_json: Value = serde_json::from_str(
            &String::from_utf8(BASE64_STANDARD.decode(patch_base64).unwrap()).unwrap(),
        )
        .unwrap();

        let expected_patch = json!([{
            "op": "add",
            "path": "/spec/template/spec/runtimeClassName",
            "value": "edera",
        }]);

        assert_eq!(patch_json, expected_patch);
    }

    #[tokio::test]
    async fn test_mutate_endpoint() {
        let filter = warp::post()
            .and(warp::path("mutate"))
            .and(warp::body::json())
            .and_then(mutate_internal);

        let admission_review = json!({
            "request": {
                "uid": "test-uid",
                "kind": {
                    "kind": "Pod"
                },
                "object": {
                    "metadata": {
                        "name": "test-name",
                        "namespace": "test-namespace"
                    }
                }
            }
        });

        let response = request()
            .method("POST")
            .path("/mutate")
            .json(&admission_review)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), 200);

        let response_body: AdmissionReviewResponse =
            serde_json::from_slice(response.body()).unwrap();

        assert_eq!(response_body.api_version, "admission.k8s.io/v1");
        assert_eq!(response_body.kind, "AdmissionReview");
        assert!(response_body.response.is_some());

        let response = response_body.response.unwrap();
        assert_eq!(response.uid, "test-uid");
        assert!(response.allowed);
        assert_eq!(response.patch_type, Some("JSONPatch".to_string()));
    }
}
