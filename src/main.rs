use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{middleware::{from_fn, Logger}, web as actweb, App, HttpResponse, HttpServer, Responder};
use paperclip::{actix::{web::{self}, OpenApiExt}, v2::models::{DefaultApiRaw, Info}};
use middleware::auth::auth_middleware;
use env_logger;
use dotenv::dotenv;
use config::{get_envar, get_officer_secret_key};

mod middleware;
mod handler;
mod config;
mod model;
mod util;

async fn healthz() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize
    dotenv().ok();
    env_logger::init();
    let required_vars = [
        "API_KEY",
        "RUST_LOG",
        "USERS",
        "OFFICER_SECRET_KEY"
    ];
    // Check each required environment variable
    for &var in required_vars.iter() {
        let _value = get_envar(var);
    }

    // Define SERVICE_PREFIX_PATH
    let service_prefix_path = std::env::var("SERVICE_PREFIX_PATH").unwrap_or_else(|_| "".to_string());

    // end of initialize
    HttpServer::new(move || {
        // Setup header swagger
        let mut spec = DefaultApiRaw::default();
        const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
        let app_version = format!("v{}", PKG_VERSION);
        spec.info = Info {
            version: app_version.into(),
            title: "Officer".into(),
            description: "<b>Serving Your Operational Needs</b> <br><br>\
            <a href=\"/auth/oidc\" target=\"_blank\">Sign in SSO</a>".to_string().into(),
            ..Default::default()
        };
        spec.base_path = Some(service_prefix_path.clone().to_string().into());
        // End of setup header swagger
        App::new()
        // Configure session middleware
        .wrap(SessionMiddleware::new(
            CookieSessionStore::default(), get_officer_secret_key().clone())
        )
        .route("/", {
            let service_prefix_path = service_prefix_path.clone();
            actweb::get().to(move || {
                let service_prefix_path = service_prefix_path.clone();
                async move {
                    HttpResponse::Found()
                        .append_header(("Location", format!("{0}/api/docs/index.html?url={0}/api/spec/v3", service_prefix_path)))
                        .finish()
                }
            })
        })
        .service(
            actweb::resource("/healthz")
            .route(actweb::get().to(healthz))
        )
        .service(
            actweb::resource("/private")
            .wrap(from_fn(auth_middleware))
            .route(actweb::get().to(healthz))
        )
        .route("/auth/oidc", actweb::get().to(handler::oidc::oauth2_login))
        .route("/auth/oidc/callback", actweb::get().to(handler::oidc::oauth2_callback))
        .service(
            web::resource("/isolate-pod")
                // .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::isolate_pod))
        )
        // Record services and routes from this line.
        .wrap_api_with_spec(spec)
        // Add routes like you normally do...
        .service(
            web::resource("/get-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::get().to(handler::kubernetes::get_pod))
        )
        .service(
            web::resource("/restart-service-deployment")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::restart_service_deployment))
        )
        .service(
            web::resource("/seed-service-deployment")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::seed_service))
        )
        .service(
            web::resource("/deploy-service")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::deploy_service))
        )
        .service(
            web::resource("/unisolate-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::unisolate_pod))
        )
        // Or just .service(echo_pet) if you're using the macro syntax
        // Mount the v2/Swagger JSON spec at this path.
        // .with_json_spec_at("/api/spec/v2")
        // If you added the "v3" feature, you can also include

        // .with_json_spec_v3_at(&format!("{0}/api/spec/v3", service_prefix_path.clone()))
        // .with_swagger_ui_at(&format!("{0}/api/docs", service_prefix_path.clone()))
        .with_json_spec_v3_at("/api/spec/v3")
        .with_swagger_ui_at("/api/docs")

        // ... or if you wish to build the spec by yourself...

        // .with_raw_json_spec(|app, spec| {
        //     app.route("/api/spec", web::get().to(move || {
        //         let spec = spec.clone();
        //         async move {
        //             paperclip::actix::HttpResponseWrapper(actix_web::HttpResponse::Ok().json(&spec))
        //         }
        //     }))
        // })
        // IMPORTANT: Build the app!
        .wrap(Logger::default())
        .build()
    }
    ).bind("0.0.0.0:8000")?
    .run().await
}