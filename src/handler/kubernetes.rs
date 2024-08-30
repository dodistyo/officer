use actix_web::{web, Error, HttpRequest};
use kube::{api::{ListParams, Patch, PatchParams}, Api, Client};
use k8s_openapi::api::core::v1::Pod;
use paperclip::actix::{api_v2_operation, web::Json, Apiv2Schema};
use serde::{Deserialize, Serialize};
use serde_json::json;
// use serde_json::Value;
use crate::config::get_api_key;
use log::info;

#[derive(Serialize, Apiv2Schema)]
pub struct SuccessResponse {
    pub status: &'static str,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
#[derive(Debug)]
pub struct PodInfo {
    name: String,
    status: String
}

#[api_v2_operation]
pub async fn get_pod(req: HttpRequest, path: web::Path<String>) -> Result<Json<Vec<PodInfo>>, Error> {
    // Load API key from environment variable
    let api_key_env = get_api_key();

    // Retrieve API key from headers
    let api_key_header = req.headers().get("x-api-key");

    // Check API key
    if api_key_header.is_none() || api_key_header.unwrap().to_str().unwrap_or("") != api_key_env {
        return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
    }

    // Interact with k8s
    // Initialize the Kubernetes client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(format!("Kubernetes connection failed: {}", e))),
    };

    // Specify the namespace
    let namespace = if path.is_empty() { "default".to_string() } else { path.into_inner() };

    // Create an API handle for Pod resources
    let pods: Api<Pod> = Api::namespaced(client, &namespace);

    // List pods with default parameters
    match pods.list(&ListParams::default()).await {
        Ok(pod_list) => {
            let pod_info: Vec<PodInfo> = pod_list.items.into_iter().map(|p| PodInfo {
                name: p.metadata.name.unwrap_or_default(),
                status: p.status.as_ref().and_then(|status| status.phase.clone()).unwrap_or_else(|| "Unknown".to_string())
            }).collect();

            info!("{:?}", pod_info);

            Ok(Json(pod_info))
        },
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not get pod: {}", e)))
    }
}

#[api_v2_operation]
pub async fn isolate_pod(_: HttpRequest) -> Result<Json<SuccessResponse>, Error> {
    let namespace = "sample"; // Replace with your namespace
    let pod_name = "sample-app-c95bd7848-bnc9n"; // Replace with your pod name
    // Interact with k8s
    // Initialize the Kubernetes client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(format!("Kubernetes connection failed: {}", e))),
    };
    // Create an API handle for Pod resources
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let patch = json!({
        "metadata": {
            "labels": {
                "isolate": "true"
            }
        }
    });
     // Apply the patch to the pod
     let pp = PatchParams::apply("add-label-isolate");
     match pods.patch(pod_name, &pp, &Patch::Merge(&patch)).await {
         Ok(_) => Ok(Json(SuccessResponse { status: "Pod isolated succesfully" })),
         Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not patch pod: {}", e)))
     }
}

#[api_v2_operation]
pub async fn unisolate_pod(_: HttpRequest) -> Result<Json<SuccessResponse>, Error> {
    let namespace = "sample"; // Replace with your namespace
    let pod_name = "sample-app-c95bd7848-bnc9n"; // Replace with your pod name
    // Interact with k8s
    // Initialize the Kubernetes client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(format!("Kubernetes connection failed: {}", e))),
    };
    // Create an API handle for Pod resources
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    let patch = json!({
        "metadata": {
            "labels": {
                "isolate": null
            }
        }
    });
     // Apply the patch to the pod
     let pp = PatchParams::apply("add-label-isolate");
     match pods.patch(pod_name, &pp, &Patch::Merge(&patch)).await {
         Ok(_) => Ok(Json(SuccessResponse { status: "Pod is being freed" })),
         Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not patch pod: {}", e)))
     }
}