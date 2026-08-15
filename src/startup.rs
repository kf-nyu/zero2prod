//! src/startup.rs
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
// use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the pool using web::Data, which boils down to an Arc samrt pointer
    let db_pool = Data::new(db_pool);
    // Capture 'connection' from the surrounding environment
    let server = HttpServer::new(move || {
        App::new()
            // Middlewares are added using the 'wrap' methode on 'App'
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            // A new entry in our routing table for POST /subscriptions requests
            .route("/subscriptions", web::post().to(subscribe))
            // REgister the connection as part of the application state
            .app_data(db_pool.clone())
    })
    .workers(4) // To avoid use all CPUs
    .listen(listener)?
    .run();
    //No .await here!
    Ok(server)
}
