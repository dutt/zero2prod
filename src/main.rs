use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cfg = get_configuration().expect("Failed to read config");
    let address = format!("127.0.0.1:{}", cfg.application_port);
    let pool = PgPool::connect(&cfg.database.connection_string())
        .await
        .expect("Failed to connect to postgres");
    let listener = TcpListener::bind(address).expect("Failed to bind");

    run(listener, pool).unwrap().await
}
