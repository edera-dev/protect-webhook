use anyhow::{Ok, Result};
use base64::prelude::*;
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
    name: String,
    namespace: String,
    runtimeclass: String, 
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

pub fn handler() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let base_path = warp::path!("mutate");

    base_path
        .and(warp::post())
        .and(warp::body::json())
        .and_then(mutate_internal)
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
        ))
    };
    // Only do an update operation if there is
    // no class set already 
    if request.runtimeclass.is_empty() {
        let patch = r#"[{
            "op": "add",
            "path": "/spec/runtimeClassName",
            "value": "edera"
        }]"#;
        let patch = BASE64_STANDARD.encode(patch);

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

        info!("mutating {}/{}", request.namespace, request.name);
        debug!("payload {:?}", response);
        return Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    } else {
        // schristoff(TODO): is it ok?
        return Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use warp::test::request;
    use warp::Reply;

    #[tokio::test]
    async fn test_mutate_function() {
        let admission_review = AdmissionReview {
            request: Some(AdmissionRequest {
                uid: "test-uid".to_string(),
                name: "test-name".to_string(),
                namespace: "test-namespace".to_string(),
                runtimeclass: String::new(),
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

        // Decode and verify the patch
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
    async fn test_mutate_endpoint() {
        let filter = warp::post()
            .and(warp::path("mutate"))
            .and(warp::body::json())
            .and_then(mutate_internal);

        let admission_review = json!({
            "request": {
                "uid": "test-uid",
                "name": "test-name",
                "namespace": "test-namespace",
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
