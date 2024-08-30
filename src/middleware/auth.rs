use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use actix_web_lab::middleware::Next;
use crate::config::get_api_key;
 
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Do something with the request here
    let api_key_env = get_api_key();

    // Retrieve API key from headers
    let api_key_header = req.headers().get("x-api-key");

    // Check API key
    if api_key_header.is_none() || api_key_header.unwrap().to_str().unwrap_or("") != api_key_env {
        return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
    }
    next.call(req).await
}
 



// pub fn auth_middleware(req: HttpRequest, srv: Service) -> impl futures_util::Future<Output = Result<ServiceResponse, ServiceError>> {
//     let api_key_env = get_api_key();

//     // Retrieve API key from headers
//     let api_key_header = req.headers().get("x-api-key");

//     // Check API key
//     if api_key_header.is_none() || api_key_header.unwrap().to_str().unwrap_or("") != api_key_env {
//         return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
//     }
//     // srv.call(req).map(|res| {
//     //     res
//     // })
// }