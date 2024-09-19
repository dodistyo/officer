use actix_web::{middleware::from_fn, App, HttpResponse, HttpServer, Responder, web as actweb};
use paperclip::{actix::{web::{self}, OpenApiExt}, v2::models::{DefaultApiRaw, Info}};
use middleware::auth::auth_middleware;
use env_logger;
use dotenv::dotenv;
use config::get_envar;

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
    let required_vars = ["API_KEY", "RUST_LOG"];
    // Check each required environment variable
    for &var in required_vars.iter() {
        let _value = get_envar(var);
    }
    // end of initialize

    HttpServer::new(move || {
        let mut spec = DefaultApiRaw::default();
        spec.info = Info {
            version: "v1.0.1".into(),
            title: "Officer Service".into(),
            description: "Serving your needs...".to_string().into(),
            ..Default::default()
        };
        App::new()
        .service(
            actweb::resource("/healthz")
            .route(actweb::get().to(healthz))
        )
        .service(
            web::resource("/isolate-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::isolate_pod))
        )
        // Record services and routes from this line.
        .wrap_api_with_spec(spec)
        // Add routes like you normally do...
        .service(
            web::resource("/deploy-service")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::deploy_service))
        )
        .service(
            web::resource("/get-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::get().to(handler::kubernetes::get_pod))
        )
        .service(
            web::resource("/unisolate-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::unisolate_pod))
        )
        .service(
            web::resource("/restart-service-deployment")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::restart_service_deployment))
        )
        // Or just .service(echo_pet) if you're using the macro syntax
        // Mount the v2/Swagger JSON spec at this path.
        // .with_json_spec_at("/api/spec/v2")
        // If you added the "v3" feature, you can also include
        .with_json_spec_v3_at("/api/spec/v3")
        .with_swagger_ui_at("/swagger")
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
        // .wrap(Logger::default())
        .build()
    }
    ).bind("0.0.0.0:8000")?
    .run().await
}