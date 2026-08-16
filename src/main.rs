//! main.rs
// use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

/// Compose multiple layers into a 'tracing''s subscriber.
///
/// # Implementation Notes - telemetry.rs
///
/// We are using 'impl Subscribe' as return type ato avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriberis
/// 'Send' and 'Sync' to make it possible to pass it to 'init_subscriber'
/// later on.

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .connect_lazy_with(configuration.database.connection_options());
//      .await
//        .expect("Failed to connect to Postgres connection pool.");
    // We have removed the hard-coded '8000' - it's now coming from our settings!
    let address = format!(
        "{}:{}", 
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
