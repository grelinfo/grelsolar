// Integration test for the app function of grelsolar
use grelsolar::app;
use tokio::time::{Duration, sleep, timeout};

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
            let app_handle = tokio::spawn(async { app().await });

            // Let the app run for a short time to ensure it starts properly
            sleep(Duration::from_millis(100)).await;

            // Check that the app task is still running
            assert!(!app_handle.is_finished(), "App should be running");

            // Terminate the app task
            app_handle.abort();

            // Wait for the task to be aborted with a timeout
            let result = timeout(Duration::from_secs(5), app_handle).await;

            // The task should have been aborted
            match result {
                Ok(Err(join_error)) if join_error.is_cancelled() => (), // Expected: task aborted
                _ => panic!("App task was not aborted as expected"),
            }
        },
    )
    .await;
}
