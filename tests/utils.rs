pub mod utils {
    use std::net::TcpListener;

    use once_cell::sync::Lazy;
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
        let mut connection = PgConnection::connect_with(&config.without_db())
            .await
            .expect("Failed to connect to Postgres");
        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create database.");
        // Migrate database
        let connection_pool = PgPool::connect_with(config.with_db())
            .await
            .expect("Failed to connect to Postgres.");
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");
        connection_pool
    }
}
