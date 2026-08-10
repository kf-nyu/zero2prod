//! main.rs
//! use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use zero2prod::run;
use std::net::TcpListener;

// async fn health_check(req: HttpRequest) -> impl Responder {
//;async fn health_check() -> impl Responder {
    //let name = req.match_info().get("name").unwrap_or("world");

    // format!("Hello {}!", &name)
    //HttpResponse::Ok().finish()
//    HttpResponse::Ok()
//}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
//    HttpServer::new(|| {
 //       App::new()
//            .route("/", web::get().to(greet))
//            .route("/{name}", web::get().to(greet))
//            .route("/health_check", web::get().to(health_check))
//    })
//    .bind("127.0.0.1:8000")?
//    .run()
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    run(listener)?.await
}
