//! src/lib.rs

use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use std::net::TcpListener;
// async fn health_check(req: HttpRequest) -> impl Responder {
async fn health_check() -> HttpResponse {
    //let name = req.match_info().get("name").unwrap_or("world");

    // format!("Hello {}!", &name)
    //HttpResponse::Ok().finish()
    HttpResponse::Ok().finish()
}

// #[tokio::main]
// Notice the different singnature!
// We return 'Server' on the happy path and we dropped the 'async' keyword
// We have no .await cal, so itis not needed anymore
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
            App::new()
//            .route("/", web::get().to(greet))
//            .route("/{name}", web::get().to(greet))
                .route("/health_check", web::get().to(health_check))
        })
        .listen(listener)?
        .run();
    //No .await here!
    Ok(server)
}
