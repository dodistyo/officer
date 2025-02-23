use std::{env, sync::Mutex, thread::sleep};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::time::Duration;

static VERIFICATION_KEY_CACHE: Lazy<Mutex<Option<DecodingKey>>> = Lazy::new(|| {
    Mutex::new(None)
});
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub exp: usize,
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
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    
    println!("Fetching JWKS from URL: {}", jwks_url);

    let mut retries = 3;
    while retries > 0 {
        let response = client.get(jwks_url).send().await;
        match response {
            Ok(resp) => {
                let jwks: Jwks = resp.json().await?;
                return Ok(jwks);
            }
            Err(err) => {
                println!("Failed to fetch JWKS: {:?}", err);
                retries -= 1;
                if retries > 0 {
                    sleep(Duration::from_secs(2));
                } else {
                    return Err(Box::new(err));
                }
            }
        }
    }

    Err("Failed to fetch JWKS after retries".into())
}

fn construct_verification_key(jwks: &Jwks, kid: &str) -> Option<DecodingKey> {
    for key in &jwks.keys {
        if key.kid == kid {
            return Some(DecodingKey::from_rsa_components(&key.n, &key.e).unwrap());
        }
    }
    None
}
async fn retrieve_verification_key(token: &str) -> Result<DecodingKey, Box<dyn std::error::Error>> {
    // let verification_key_cache = VERIFICATION_KEY_CACHE.read().map_err(|_| "Failed to acquire read lock")?;
    let mut verification_key_cache = VERIFICATION_KEY_CACHE.lock().map_err(|_| "Failed to acquire lock")?;
    if verification_key_cache.is_none() {
        let header = decode_header(token)?;
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
        
        let verification_key = match construct_verification_key(&jwks, &kid) {
            Some(key) => key,
            None => {
                println!("No matching key found in JWKS for kid: {}", kid);
                return Err("No matching key found in JWKS".into());
            }
        };
        *verification_key_cache = Some(verification_key.clone());
        Ok(verification_key)
    } else {
        Ok(verification_key_cache.clone().unwrap())
    }
    // } else {
    //     Ok(verification_key_cache.clone().unwrap())
    // }
}

// Validate a JWT token
pub async fn validate_token(token: &str) -> Result<TokenData<Claims>, Box<dyn std::error::Error>> {
    let mut validation = Validation::new(Algorithm::RS256);
    let oauth2_client_id = std::env::var("OAUTH2_CLIENT_ID").expect("OAUTH2_CLIENT_ID environment variable not set");
    validation.set_audience(&[format!("{}", oauth2_client_id)]);
    validation.validate_exp = true;
    let verification_key    = retrieve_verification_key(token).await?; 
    let token_data = match decode::<Claims>(token, &verification_key, &validation) {
        Ok(data) => data,
        Err(err) => {
            println!("Failed to decode token: {:?}", err);
            return Err("Failed to decode token".into());
        }
    };
    Ok(token_data)
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