//! src/lib.rs

// use actix_web::dev::Server;
use actix_web::HttpResponse;
// use std::net::TcpListener;

// async fn health_check(req: HttpRequest) -> impl Responder {
pub async fn health_check() -> HttpResponse {
    //let name = req.match_info().get("name").unwrap_or("world");

    // format!("Hello {}!", &name)
    //HttpResponse::Ok().finish()
    HttpResponse::Ok().finish()
}
