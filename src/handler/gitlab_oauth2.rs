use actix_web::{web, HttpResponse, Responder};
use log::{error, info};
use oauth2::{basic::BasicClient, http::{HeaderMap, Method}, reqwest::async_http_client, AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl};
use actix_session::Session;
use serde::Deserialize;
use serde_json::json;
use url::Url;

#[allow(unused)]
#[derive(Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct Identity {
    #[allow(unused)]
    pub provider: String,
    #[allow(unused)]
    pub extern_uid: String
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct UserInfo {
    id: u64,
    name: String,
    username: String,
    email: String,
    commit_email: String,
    is_admin: bool,
    identities: Vec<Identity>,
    avatar_url: String
}

fn gitlab_oauth_client() -> BasicClient {
    let oauth2_gitlab_url = std::env::var("OAUTH2_GITLAB_URL").expect("OAUTH2_GITLAB_URL environment variable not set");
    let oauth2_gitlab_client_id = std::env::var("OAUTH2_GITLAB_CLIENT_ID").expect("OAUTH2_GITLAB_CLIENT_ID environment variable not set");
    let oauth2_gitlab_client_secret = std::env::var("OAUTH2_GITLAB_CLIENT_SECRET").expect("OAUTH2_GITLAB_CLIENT_SECRET environment variable not set");
    let oauth2_redirect_url = std::env::var("OAUTH2_REDIRECT_URL").expect("OAUTH2_GITLAB_CLIENT_SECRET environment variable not set");
    let auth_url = AuthUrl::new(
        format!("{}/oauth/authorize", oauth2_gitlab_url),
    ).expect("Invalid authorization endpoint URL");

    let token_url = TokenUrl::new(
        format!("{}/oauth/token", oauth2_gitlab_url),
    ).expect("Invalid token endpoint URL");

    let redirect_url = RedirectUrl::new(
        oauth2_redirect_url,
    ).expect("Invalid redirect URL");
    
    let client_id = ClientId::new(oauth2_gitlab_client_id);
    let client_secret = ClientSecret::new(oauth2_gitlab_client_secret);

    BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(redirect_url)
}

pub async fn oauth_login(session: Session) -> impl Responder {
    let client = gitlab_oauth_client();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read_user".to_string())) // GitLab's scope
        .add_scope(Scope::new("email".to_string())) // GitLab's scope
        .add_scope(Scope::new("profile".to_string())) // GitLab's scope
        .url();

    // Store CSRF token in session
    session.insert("csrf_token", csrf_token.secret().as_str()).unwrap();

    // Redirect user to GitLab's authorization URL
    HttpResponse::Found().append_header(("LOCATION", auth_url.to_string())).finish()
}

async fn read_user(api_base_url: &str, access_token: &AccessToken) -> Result<UserInfo, String> {
    let url = Url::parse(&format!(
        "{}/user?access_token={}",
        api_base_url,
        access_token.secret()
    )).map_err(|e| format!("Invalid URL: {}", e))?;

    let request = oauth2::HttpRequest {
        url,
        method: Method::GET,
        headers: HeaderMap::new(),
        body: Vec::new(),
    };

    match async_http_client(request).await {
        Ok(resp) => {
            serde_json::from_slice(&resp.body)
                .map_err(|e| format!("Failed to parse response: {}", e))
        }
        Err(e) => {
            let error_message = format!("Failed to retrieve user info: {}", e);
            info!("{}", error_message);
            Err(error_message)
        }
    }
}

pub async fn oauth_callback(
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
    
    // Create your oauth client
    let client = gitlab_oauth_client();

    // Build the token request
    let token_request = client.exchange_code(AuthorizationCode::new(query.code.clone()));

    // Send the request using reqwest
    match token_request.request_async(async_http_client).await {
        Ok(token_response) => {
            // Assuming the access token is in the response
            let access_token = token_response.access_token().secret();
            let user_info = read_user("https://git.ihc.id/api/v4", token_response.access_token()).await;
            info!("{:?}", user_info);
            HttpResponse::Ok().json(json!({"access_token": access_token}))
        }
        Err(e) => {
            error!("Failed to exchange code for token: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to get access token"}))
        }
    }
}