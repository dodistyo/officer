use std::{env, process};
use dotenv::dotenv;

pub fn get_api_key() -> String {
    dotenv().ok();  // Load environment variables from .env file
    env::var("API_KEY").expect("API_KEY must be set")
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