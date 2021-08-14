use std::net::TcpListener;

use zero2prod::startup::run;
use zero2prod::configuration::get_configuration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
   let cfg = get_configuration().expect("Failed to read config");
   let address = format!("127.0.0.1:{}", cfg.application_port);
   let listener = TcpListener::bind(address)
        .expect("Failed to bind");

    run(listener).unwrap().await
}
