use std::env;
use dotenv::dotenv;

pub fn get_api_key() -> String {
    dotenv().ok();  // Load environment variables from .env file
    env::var("API_KEY").expect("API_KEY must be set")
}
