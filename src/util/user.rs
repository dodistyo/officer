use jsonwebtoken::TokenData;

use crate::{
    model::auth::AuthJwtHeader, util::jwt::{extract_token_from_header, validate_token}
};

use super::jwt::Claims;

pub async fn user_data(auth_jwt_header: AuthJwtHeader) -> Result<TokenData<Claims>, actix_web::Error> {
    let token = extract_token_from_header(auth_jwt_header).await?;
    let token_data = validate_token(&token).await?;
    Ok(token_data)
}