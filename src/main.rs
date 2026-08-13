//! main.rs
//! use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
// async fn health_check(req: HttpRequest) -> impl Responder {
//;async fn health_check() -> impl Responder {
//let name = req.match_info().get("name").unwrap_or("world");

// format!("Hello {}!", &name)
//HttpResponse::Ok().finish()
//    HttpResponse::Ok()
//}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    //Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    // We have removed the hard-coded '8000' - it's now coming from our settings!
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
