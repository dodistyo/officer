use std::str;

use jsonwebtoken::{Header, TokenData };
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::{
    model::auth::AuthJwtHeader, util::jwt::extract_token_from_header
};

use super::jwt::Claims;

pub async fn user_data(auth_jwt_header: AuthJwtHeader) -> Result<TokenData<Claims>, actix_web::Error> {
    let token = extract_token_from_header(auth_jwt_header).await?;
    // Split the JWT into its parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(actix_web::error::ErrorBadRequest("Invalid JWT format"));
    }

    // Decode the header (first part)
    let header = parts[0];
    let header_decoded = URL_SAFE_NO_PAD.decode(header).map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    let header_str = str::from_utf8(&header_decoded)?;
    let header: Header = serde_json::from_str(header_str)?;

    // Decode the payload (second part)
    let payload = parts[1];
    let payload_decoded = URL_SAFE_NO_PAD.decode(payload).map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    let payload_str = str::from_utf8(&payload_decoded)?;
    let claims: Claims = serde_json::from_str(payload_str)?;

    // Construct TokenData
    let token_data = TokenData {
        header,
        claims,
    };
    Ok(token_data)
}