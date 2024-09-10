use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use paperclip::actix::{Apiv2Schema, Apiv2Security};
use serde::{Deserialize, Serialize};
// Swagger Auth
#[derive(Apiv2Security)]
#[openapi(
  apiKey,
  in = "header",
  name = "x-api-key",
  description = "API key"
)]
pub struct AuthHeader(pub String);

impl FromRequest for AuthHeader {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract the custom header value from the request
        if let Some(header_value) = req.headers().get("x-api-key") {
            if let Ok(header_str) = header_value.to_str() {
                return ready(Ok(AuthHeader(header_str.to_string())));
            }
        }
        // If the header is not present or not valid, return an error
        ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized")))
    }
}

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
    pub status: String
}

#[derive(Serialize, Deserialize, Apiv2Schema)]
pub struct ServiceDeploymentPayload {
    pub namespace: String,
    pub service_deployment: String
}