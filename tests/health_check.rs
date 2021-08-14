use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{net::TcpListener, vec};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

#[actix_rt::test]
async fn health_check() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

#[actix_rt::test]
async fn subscriptions_200() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40foomail.com";

    let resp = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to request");

    assert_eq!(200, resp.status().as_u16());

    let record = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.pool)
        .await
        .expect("Failed to fetch subscription");

    assert_eq!(record.email, "ursula_le_guin@foomail.com");
    assert_eq!(record.name, "le guin");
}

#[actix_rt::test]
async fn subscriptions_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let cases = vec![
        ("email=ursula_le_guin%40foomail.com", "name missing"),
        ("name=le%20guin", "email missing"),
        ("", "both missing"),
    ];
    for (body, message) in cases {
        let resp = client
            .post(&format!("{}/subscriptions", app.address))
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

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut cfg = get_configuration().expect("Failed to read config");
    cfg.database.dbname = Uuid::new_v4().to_string();

    let pool = setup_dbpool(&cfg.database).await;

    let server = zero2prod::startup::run(listener, pool.clone()).expect("failed to create server");
    let _ = tokio::spawn(server);

    TestApp { address, pool }
}

pub async fn setup_dbpool(cfg: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&cfg.connection_string_without_db())
        .await
        .expect("Failed to connect to db");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, cfg.dbname))
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&cfg.connection_string())
        .await
        .expect("Failed to connect pool to postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    pool
}
