use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};
use paperclip::actix::Apiv2Security;
// Swagger Auth
#[derive(Apiv2Security)]
#[openapi(
  apiKey,
  alias = "JWT Token",
  in = "header",
  name = "Authorization",
  description = "JWT Auth, example: \"Bearer thetoken\""
)]
pub struct AuthJwtHeader(pub String);
#[derive(Apiv2Security)]
#[openapi(
  apiKey,
  alias = "API Key",
  in = "header",
  name = "X-API-KEY",
  description = "API key"
)]
pub struct ApiKeyHeader(pub String);

impl FromRequest for ApiKeyHeader {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract the custom header value from the request
        if let Some(header_value) = req.headers().get("X-API-KEY") {
            if let Ok(header_str) = header_value.to_str() {
                return ready(Ok(ApiKeyHeader(header_str.to_string())));
            }
        }
        // If the header is not present or not valid, return an error
        ready(Ok(ApiKeyHeader("".to_owned())))
    }
}

impl FromRequest for AuthJwtHeader {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Extract the custom header value from the request
        if let Some(header_value) = req.headers().get("Authorization") {
            if let Ok(header_str) = header_value.to_str() {
                return ready(Ok(AuthJwtHeader(header_str.to_string())));
            }
        }
        // If the header is not present or not valid, return an error
        ready(Ok(AuthJwtHeader("".to_owned())))
    }
}