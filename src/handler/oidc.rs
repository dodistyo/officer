use actix_web::{web, HttpResponse, Responder};
use log::error;
use oauth2::{basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use actix_session::Session;
use serde::Deserialize;
use serde_json::json;

use crate::util::jwt::validate_token;

#[derive(Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize, Debug)]
pub struct Identity {
    #[allow(unused)]
    pub provider: String,
    #[allow(unused)]
    pub extern_uid: String
}

fn oauth2_client() -> BasicClient {
    let oauth2_url = std::env::var("OAUTH2_URL").expect("OAUTH2_URL environment variable not set");
    let oauth2_client_id = std::env::var("OAUTH2_CLIENT_ID").expect("OAUTH2_CLIENT_ID environment variable not set");
    let oauth2_client_secret = std::env::var("OAUTH2_CLIENT_SECRET").expect("OAUTH2_CLIENT_SECRET environment variable not set");
    let oauth2_redirect_url = std::env::var("OAUTH2_REDIRECT_URL").expect("OAUTH2_CLIENT_SECRET environment variable not set");
    let auth_url = AuthUrl::new(
        format!("{}/authorize", oauth2_url),
    ).expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new(
        format!("{}/token", oauth2_url),
    ).expect("Invalid token endpoint URL");

    let redirect_url = RedirectUrl::new(
        oauth2_redirect_url,
    ).expect("Invalid redirect URL");
    
    let client_id = ClientId::new(oauth2_client_id);
    let client_secret = ClientSecret::new(oauth2_client_secret);

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}

pub async fn oauth2_login(session: Session) -> impl Responder {
    let client = oauth2_client();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string())) 
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(
            Scope::new(
                format!("{}/.default", std::env::var("OAUTH2_CLIENT_ID").expect("OAUTH2_CLIENT_ID environment variable not set"))
            )
        )
        .url();

    // Store CSRF token in session
    session.insert("csrf_token", csrf_token.secret().as_str()).unwrap();

    // Redirect user to GitLab's authorization URL
    HttpResponse::Found().append_header(("LOCATION", auth_url.to_string())).finish()
}

pub async fn oauth2_callback(
    session: Session,
    query: web::Query<OAuthQuery>,
) -> impl Responder {
    let csrf_token = match session.get::<String>("csrf_token") {
        Ok(Some(token)) => token,
        _ => return HttpResponse::BadRequest().body("Invalid CSRF token"),
    };

    if query.state != csrf_token {
        return HttpResponse::BadRequest().body("Invalid CSRF token");
    }
    
    let client = oauth2_client();
    // let oauth2_user_info_endpoint = std::env::var("OAUTH2_USER_INFO_ENDPOINT").expect("OAUTH2_USER_INFO_ENDPOINT environment variable not set");

    let token_request = client.exchange_code(AuthorizationCode::new(query.code.clone()));

    match token_request.request_async(async_http_client).await {
        Ok(token_response) => {
            let access_token = token_response.access_token().secret();
            let token_data = validate_token(&access_token).await.unwrap();
            let users = std::env::var("USERS").expect("USERS environment variable not set");
            let user_db: Vec<&str> = users.split(',').collect();
            if user_db.contains(&token_data.claims.upn.as_str()) {
               HttpResponse::Ok().json(json!({"token": access_token}))
            } else {
                HttpResponse::Unauthorized().json(json!({"error": "Unauthorized!"}))
            }
        }
        Err(e) => {
            error!("Failed to exchange code for token: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to get access token"}))
        }
    }
}