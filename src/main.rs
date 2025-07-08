//! grelsolar - A Rust application for solar energy management

use grelsolar::app;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = app().await {
        eprintln!("Application error: {e:?}");
        std::process::exit(1);
    }
    std::process::exit(0);
}
