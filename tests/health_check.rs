use std::net::TcpListener;

#[actix_rt::test]
async fn health_check() {
    let host = spawn_app();

    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/health_check", host))
        .send()
        .await
        .expect("Failed to request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("failed to create server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
