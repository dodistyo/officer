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
    println!("Token header: {:?}", header);
    let kid = match header.kid {
        Some(kid) => kid,
        None => {
            println!("No kid in JWT header");
            return Err("No kid in JWT header".into());
        }
    };
    
    let jwks_url = get_jwks_url();
    let jwks = match fetch_jwks(&jwks_url).await {
        Ok(jwks) => jwks,
        Err(err) => {
            println!("Failed to fetch JWKS: {:?}", err);
            return Err("Failed to fetch JWKS".into());
        }
    };
    
    let decoding_key = match get_decoding_key(&jwks, &kid) {
        Some(key) => key,
        None => {
            println!("No matching key found in JWKS for kid: {}", kid);
            return Err("No matching key found in JWKS".into());
        }
    };

    let mut validation = Validation::new(Algorithm::RS256);
    let oauth2_client_id = std::env::var("OAUTH2_CLIENT_ID").expect("OAUTH2_CLIENT_ID environment variable not set");
    validation.set_audience(&[format!("{}", oauth2_client_id)]);
    validation.validate_exp = true;

    let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(data) => data,
        Err(err) => {
            println!("Failed to decode token: {:?}", err);
            return Err("Failed to decode token".into());
        }
    };
    Ok(token_data)
}
