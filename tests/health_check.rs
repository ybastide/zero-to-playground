use crate::utils::utils::spawn_app;

mod utils;

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let resp = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Request failed.");
    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}
