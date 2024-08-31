use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, Error
};
// use actix_web_lab::middleware::Next;
use crate::{config::get_api_key, model::kubernetes::AuthHeader};

use actix_web::middleware::Next;

pub async fn auth_middleware(
    auth_header: AuthHeader,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    let api_key_env = get_api_key();

    // Retrieve API key from headers
    let api_key_header = auth_header.0.as_str();

    // Check API key
    if api_key_header != api_key_env {
        return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
    }
    // invoke the wrapped middleware or service
    let res = next.call(req).await?;

    // post-processing

    Ok(res)
}
 
// pub async fn auth_middleware(
//     req: ServiceRequest,
//     next: Next<impl MessageBody>,
// ) -> Result<ServiceResponse<impl MessageBody>, Error> {
//     // Do something with the request here
//     let api_key_env = get_api_key();

//     // Retrieve API key from headers
//     let api_key_header = req.headers().get("x-api-key");

//     // Check API key
//     if api_key_header.is_none() || api_key_header.unwrap().to_str().unwrap_or("") != api_key_env {
//         return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
//     }
//     next.call(req).await
// }