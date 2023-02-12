use crate::utils::*;

pub mod utils {
    use std::net::TcpListener;

    use once_cell::sync::Lazy;
    use secrecy::ExposeSecret;
    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use uuid::Uuid;

    use zero2prod::configuration::{get_configuration, DatabaseSettings};
    use zero2prod::telemetry::{get_subscriber, init_subscriber};

    static TRACING: Lazy<()> = Lazy::new(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        };
    });

    pub struct TestApp {
        pub address: String,
        pub db_pool: PgPool,
    }

    pub async fn spawn_app() -> TestApp {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        Lazy::force(&TRACING);

        let listener = TcpListener::bind("127.0.0.1:0").expect("Cannot bind listener");
        let mut configuration = get_configuration().expect("Failed to read configuration");
        configuration.database.database_name = Uuid::new_v4().to_string();
        let connection_pool = configure_database(&configuration.database).await;
        let port = listener.local_addr().unwrap().port();
        let server = zero2prod::startup::run(listener, connection_pool.clone())
            .expect("Failed to start server");
        let _ = tokio::spawn(server);
        TestApp {
            address: format!("http://localhost:{port}"),
            db_pool: connection_pool,
        }
    }

    pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
        // Create database
        let mut connection =
            PgConnection::connect(&config.connection_string_without_db().expose_secret())
                .await
                .expect("Failed to connect to Postgres");
        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create database.");
        // Migrate database
        let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");
        connection_pool
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let resp = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Request failed.");
    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_20x_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(reqwest::StatusCode::CREATED, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}