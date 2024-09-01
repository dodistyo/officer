use actix_web::{middleware::from_fn, App, HttpServer};
use paperclip::actix::{web::{self}, OpenApiExt};
use middleware::auth::auth_middleware;
use env_logger;
use dotenv::dotenv;
use config::get_envar;

mod middleware;
mod handler;
mod config;
mod model;

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

    HttpServer::new(|| App::new()
        // Record services and routes from this line.
        .wrap_api()
        // Add routes like you normally do...
        .service(
            web::resource("/get-pod/{namespace}")
                .wrap(from_fn(auth_middleware))
                .route(web::get().to(handler::kubernetes::get_pod))
        )
        .service(
            web::resource("/isolate-pod")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(handler::kubernetes::isolate_pod))
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
    ).bind("0.0.0.0:8000")?
    .run().await
}