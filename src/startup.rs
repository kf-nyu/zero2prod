//! src/lib.rs
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;

// Notice the different singnature!
// We return 'Server' on the happy path and we dropped the 'async' keyword
// We have no .await cal, so itis not needed anymore
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the pool using web::Data, which boils down to an Arc samrt pointer
    let db_pool = web::Data::new(db_pool);
    // Capture 'connection' from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            //            .route("/", web::get().to(greet))
            //            .route("/{name}", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            // REgister the connection as part of the application state
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    //No .await here!
    Ok(server)
}
