use actix_web::{
    body::MessageBody, dev::{ServiceRequest, ServiceResponse}, Error
};
use log::info;
// use actix_web_lab::middleware::Next;
use crate::{config::get_api_key, model::auth::{ApiKeyHeader, AuthJwtHeader}, util::jwt::validate_token};

use actix_web::middleware::Next;

pub async fn auth_middleware(
    api_key_header: ApiKeyHeader,
    auth_jwt_header: AuthJwtHeader,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Log all headers
    // for (header_name, header_value) in req.headers().iter() {
    //     info!("Header: {}: {:?}", header_name, header_value);
    // }

    // pre-processing
    let api_key_env = get_api_key();
    let api_key = api_key_header.0.as_str();
    if api_key.is_empty() {
        let jwt = auth_jwt_header.0.as_str();
        let res = next.call(req).await?;

        // Check if the header starts with "Bearer " and extract the token
        let token = if jwt.starts_with("Bearer ") {
            &jwt["Bearer ".len()..]
        } else {
            return Err(actix_web::error::ErrorUnauthorized("Invalid Bearer Token!")); // Handle the error case
        };
        println!("Token here: {:?}", token);

        match validate_token(token).await {
            Ok(token) => {
                println!("Token hore: {:?}", token);
                info!("User: {}", token.claims.sub);
                Ok(res)
            },
            Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid Token")),
        }
    } else {
        // Check API key
        if api_key != api_key_env {
            return Err(actix_web::error::ErrorUnauthorized("Invalid API key")); // Handle the error case
        }
        // invoke the wrapped middleware or service
        let res = next.call(req).await?;

        // post-processing

        Ok(res)
    }
}