use std::{env, process};
use actix_web::cookie::Key;
use dotenv::dotenv;

pub fn get_api_key() -> String {
    dotenv().ok();  // Load environment variables from .env file
    env::var("API_KEY").expect("API_KEY must be set")
}

pub fn get_officer_secret_key() -> Key {
    dotenv().ok();
    let key_string = env::var("OFFICER_SECRET_KEY").expect("OFFICER_SECRET_KEY must be set");
    let key_bytes = key_string.into_bytes();
    if key_bytes.len() < 32 {
        panic!("OFFICER_SECRET_KEY must be at least 32 bytes long");
    }
    Key::from(&key_bytes)
}

pub fn get_envar(var_name: &str) -> String {
    match env::var(var_name) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Error: {} environment variable is not set", var_name);
            process::exit(1)
        }
    }
}