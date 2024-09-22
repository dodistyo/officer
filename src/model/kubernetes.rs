
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct SuccessResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct UnisolatePodPayload {
    pub namespace: String,
    pub pod_name: String,
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct PodInfo {
    pub name: String,
    pub status: String,
    pub age: String
}

#[derive(Deserialize, Apiv2Schema)]
pub struct GetPodQuery {
    pub namespace: String
}

#[derive(Deserialize, Apiv2Schema)]
pub struct RestartServicePayload {
    pub namespace: String,
    pub service_deployment: String
}

#[derive(Deserialize, Apiv2Schema)]
pub struct DeployServicePayload {
    pub namespace: String,
    pub service_deployment: String,
    pub container_name: String,
    pub version: String
}