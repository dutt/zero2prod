use sqlx::{Connection, PgConnection};
use std::{net::TcpListener, vec};
use zero2prod::configuration::get_configuration;

#[actix_rt::test]
async fn health_check() {
    let host = spawn_app();
    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", &host))
        .send()
        .await
        .expect("Failed to request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

#[actix_rt::test]
async fn subscriptions_200() {
    let host = spawn_app();

    let config = get_configuration().expect("Failed to read config");
    let connection_string = config.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40foomail.com";

    let resp = client
        .post(&format!("{}/subscriptions", host))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to request");

    assert_eq!(200, resp.status().as_u16());

    let record = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch subscription");

    assert_eq!(record.email, "ursula_le_guin@foomail.com");
    assert_eq!(record.name, "le guin");
}

#[actix_rt::test]
async fn subscriptions_400() {
    let host = spawn_app();
    let client = reqwest::Client::new();
    let cases = vec![
        ("email=ursula_le_guin%40foomail.com", "name missing"),
        ("name=le%20guin", "email missing"),
        ("", "both missing"),
    ];
    for (body, message) in cases {
        let resp = client
            .post(&format!("{}/subscriptions", host))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to request");

        assert_eq!(
            400,
            resp.status().as_u16(),
            "test case '{}' failed",
            message
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("failed to create server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
