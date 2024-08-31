use actix_web::{web, Error, HttpRequest};
use kube::{api::{ListParams, Patch, PatchParams}, Api, Client};
use k8s_openapi::api::core::v1::Pod;
use paperclip::actix::{api_v2_operation, web::Json};
use serde_json::{json, Value};
use crate::model::kubernetes::{AuthHeader, PodInfo, SuccessResponse, UnisolatePodPayload};

#[api_v2_operation(tags("kubernetes"))]
/// Get pods in a namespace 
///
/// List all pods in a namespace, it will show their names and statuses
pub async fn get_pod(path: web::Path<String>) -> Result<Json<Vec<PodInfo>>, Error> {
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

            // info!("{:?}", pod_info);

            Ok(Json(pod_info))
        },
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not get pod: {}", e)))
    }
}

#[api_v2_operation(tags("kubernetes"))]
/// Isolate pod
///
/// Isolating pod network connection, both Ingress and Egress
/// 
/// Requirement: Network policy that deny Ingress and Eggress with label selector isolate: "true" 
/// 
/// Example usage: Use this endpoint to isolate pod when threat is detected on a pod
pub async fn isolate_pod(_: HttpRequest, payload: web::Json<Value>) -> Result<Json<SuccessResponse>, Error> {
    // Get the JSON payload
    let json_payload = payload.into_inner();
    // Extract values from the `output_fields` object
    let output_fields = json_payload.get("output_fields").and_then(Value::as_object);

    // Get `namespace` and `pod name`
    let namespace = output_fields
        .and_then(|fields| fields.get("k8s.ns.name"))
        .and_then(Value::as_str)
        .unwrap_or("Unknown");

    let pod_name = output_fields
        .and_then(|fields| fields.get("k8s.pod.name"))
        .and_then(Value::as_str)
        .unwrap_or("Unknown");

    let falco_rule = json_payload.get("rule").and_then(Value::as_str).unwrap_or("Unknown");
    let implemented_falco_rules = vec!["network_scan_process_in_container"];
    
    if implemented_falco_rules.contains(&falco_rule) {
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
    } else {
        Ok(Json(SuccessResponse { status: "Skipped, no action taken" }))
    }
   
}

#[api_v2_operation(tags("kubernetes"))]
/// Remove pod Isolation
///
/// Allowing Ingress and Egress network connection
/// 
/// Requirement: Network policy that deny Ingress and Eggress with label selector isolate: "true" 
/// 
/// Example usage: Use this endpoint to isolate pod when threat is detected 
pub async fn unisolate_pod(_: HttpRequest, payload: web::Json<UnisolatePodPayload>) -> Result<Json<SuccessResponse>, Error> {
    let namespace = &payload.namespace;
    let pod_name = &payload.pod_name;
    // Interact with k8s
    // Initialize the Kubernetes client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(format!("Kubernetes connection failed: {}", e))),
    };
    // Create an API handle for Pod resources
    let pods: Api<Pod> = Api::namespaced(client, namespace);
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