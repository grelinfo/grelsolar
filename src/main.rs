//! grelsolar - A Rust application for solar energy management
//! The application is small enough to run on a single thread,
//! making it suitable for low-resource environments.

use grelsolar::app;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = app().await {
        eprintln!("Application error: {e:?}");
        std::process::exit(1);
    }
    std::process::exit(0);
}
