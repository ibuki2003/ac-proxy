use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[get("/")]
async fn index() -> impl Responder {
    let response = format!(
        "ac-proxy {}\nhttps://github.com/ibuki2003/ac-proxy\n",
        VERSION
    );
    HttpResponse::Ok().body(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let pool =
        mysql_async::Pool::from_url(env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
            .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "*")),
            )
            .app_data(web::Data::new(ac_proxy::AppState { pool: pool.clone() }))
            .service(index)
            .service(ac_proxy::proxy_cache::proxy_service)
    })
    .bind(env::var("BIND_ADDR").unwrap_or(String::from("0.0.0.0:8080")))?
    .run()
    .await
}
