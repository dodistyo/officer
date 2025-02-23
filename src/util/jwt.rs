use std::env;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use reqwest::get;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

// Define the claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    // Add other claims you expect in the JWT
}
#[derive(Debug, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    n: String,
    e: String,
}

fn get_jwks_url() -> String {
    dotenv().ok();  // Load environment variables from .env file
    let jwks_url = env::var("OAUTH2_JWKS_URL").expect("OAUTH2_JWKS_URL must be set");
    format!("{}", jwks_url)
}

async fn fetch_jwks(jwks_url: &str) -> Result<Jwks, Box<dyn std::error::Error>> {
    let response = get(jwks_url).await?;
    let jwks: Jwks = response.json().await?;
    Ok(jwks)
}

fn get_decoding_key(jwks: &Jwks, kid: &str) -> Option<DecodingKey> {
    for key in &jwks.keys {
        if key.kid == kid {
            return Some(DecodingKey::from_rsa_components(&key.n, &key.e).unwrap());
        }
    }
    None
}
// Create a JWT token
// pub fn create_token(sub: &str) -> Result<String, jsonwebtoken::errors::Error> {
//     let claims = Claims {
//         sub: sub.to_owned(),
//         exp: 10000000000, // Set your expiration time here
//     };
//     let encoding_key = EncodingKey::from_secret(get_jwt_secret_key().as_ref());
//     encode(&Header::new(Algorithm::HS256), &claims, &encoding_key)
// }

// Validate a JWT token
pub async fn validate_token(token: &str) -> Result<TokenData<Claims>, Box<dyn std::error::Error>> {
    let header = decode_header(token)?;
    let kid = header.kid.ok_or("No kid in JWT header")?;
    let jwks_url = get_jwks_url();
    let jwks = fetch_jwks(&jwks_url).await?;
    let decoding_key = get_decoding_key(&jwks, &kid).ok_or("No matching key found in JWKS")?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    println!("Token data: {:?}", token_data);
    Ok(token_data)
}
