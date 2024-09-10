use actix_web::{web, Error};
use chrono::{DateTime, Utc};
use kube::{api::{ListParams, Patch, PatchParams}, Api, Client};
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Pod};
use paperclip::actix::{api_v2_operation, web::Json};
use serde_json::{json, Value};
use crate::{model::kubernetes::{AuthHeader, PodInfo, ServiceDeploymentPayload, SuccessResponse, UnisolatePodPayload}, util::time_helper};

#[api_v2_operation(tags("kubernetes"))]
/// Get pods in a namespace 
///
/// List all pods in a namespace, it will show their names and statuses
pub async fn get_pod(_: AuthHeader, path: web::Path<String>) -> Result<Json<Vec<PodInfo>>, Error> {
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
            let now = Utc::now();
            let pod_info: Vec<PodInfo> = pod_list.items.into_iter().map(|p| {
                let name = p.metadata.name.unwrap_or_default();
                let status = p.status.as_ref().and_then(|status| status.phase.clone()).unwrap_or_else(|| "Unknown".to_string());
                
                // Convert Kubernetes Time to chrono DateTime
                let creation_time = match p.metadata.creation_timestamp {
                    Some(ref ts) => DateTime::<Utc>::from(ts.0.clone()),
                    None => now, // Fallback if creation_timestamp is None
                };
                
                // Calculate the age
                let age_duration = now.signed_duration_since(creation_time).num_seconds();
                let age = time_helper::format_duration(age_duration);
                
                PodInfo {
                    name,
                    status,
                    age,
                }
            }).collect();

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
pub async fn isolate_pod(_: AuthHeader, payload: web::Json<Value>) -> Result<Json<SuccessResponse>, Error> {
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
            Ok(_) => Ok(Json(SuccessResponse { status: "Pod isolated succesfully".to_string() })),
            Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not patch pod: {}", e)))
        }
    } else {
        Ok(Json(SuccessResponse { status: "Skipped, no action taken".to_string() }))
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
pub async fn unisolate_pod(_: AuthHeader, payload: web::Json<UnisolatePodPayload>) -> Result<Json<SuccessResponse>, Error> {
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
         Ok(_) => Ok(Json(SuccessResponse { status: "Pod is being freed".to_string() })),
         Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not patch pod: {}", e)))
     }
}

#[api_v2_operation(tags("kubernetes"))]
/// Restart Kubernetes Deployment
///
/// This api will restart a deployment on a specific namespace
pub async fn restart_service_deployment(_: AuthHeader, payload: web::Json<ServiceDeploymentPayload>) -> Result<Json<SuccessResponse>, Error> {
    // Get `namespace` and `pod name`
    let namespace = &payload.namespace;

    let service_deployment = &payload.service_deployment;

    // Interact with k8s
    // Initialize the Kubernetes client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(format!("Kubernetes connection failed: {}", e))),
    };
    // Create an API handle for Pod resources
    let deployment: Api<Deployment> = Api::namespaced(client, &namespace);
    let patch = json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": Utc::now().to_rfc3339(),
                    }
                }
            }
        }
    });
    // Apply the patch to the pod
    let pp = PatchParams::apply("restart-deployment");
    match deployment.patch(service_deployment, &pp, &Patch::Merge(&patch)).await {
        Ok(_) => Ok(Json(SuccessResponse { status: format!("Deployment {} restarted", service_deployment) })),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Could not patch pod: {}", e)))
    }

   
}