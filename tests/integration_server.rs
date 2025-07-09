// Integration test for the server
use envconfig::Envconfig;
use grelsolar::core::config::Config;
use grelsolar::server::server;
use tokio::time::{Duration, sleep, timeout};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_start_and_stop() {
    temp_env::async_with_vars(
        [
            ("SOLARLOG_URL", Some("http://localhost:1234")),
            ("SOLARLOG_PASSWORD", Some("pw")),
            ("HOMEASSISTANT_URL", Some("http://localhost:5678")),
            ("HOMEASSISTANT_TOKEN", Some("token")),
        ],
        async {
            // Start the app function in a background task
            let token = CancellationToken::new();
            let server_token = token.clone();

            let config = Config::init_from_env().expect("cannot load config");

            let app_handle = tokio::spawn(async { server(config, server_token).await });

            // Let the app run for a short time to ensure it starts properly
            sleep(Duration::from_millis(100)).await;

            // Check that the app task is still running
            assert!(!app_handle.is_finished(), "App should be running");

            // Shutdown the app task
            token.cancel();

            // Wait for the task to finish with a timeout
            let result = timeout(Duration::from_secs(5), app_handle).await;

            // The task should have exited gracefully or been cancelled
            result
                .expect("App task did not finish in time")
                .expect("App task failed");
        },
    )
    .await;
}
