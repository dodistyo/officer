use std::env;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

// Define the claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
pub fn get_jwt_secret_key() -> String {
    dotenv().ok();  // Load environment variables from .env file
    env::var("OFFICER_SECRET_KEY").expect("OFFICER_SECRET_KEY must be set")
}
// Create a JWT token
pub fn create_token(sub: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: sub.to_owned(),
        exp: 10000000000, // Set your expiration time here
    };
    let encoding_key = EncodingKey::from_secret(get_jwt_secret_key().as_ref());
    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key)
}

// Validate a JWT token
pub fn validate_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(get_jwt_secret_key().as_ref());
    decode::<Claims>(token, &decoding_key, &Validation::new(Algorithm::HS256))
}
