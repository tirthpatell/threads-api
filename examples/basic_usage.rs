//! Basic usage example for the threads-rs crate.
//!
//! Run with:
//! ```sh
//! THREADS_ACCESS_TOKEN=your_token cargo run --example basic_usage
//! ```

use threads_rs::client::{Client, Config};

#[tokio::main]
async fn main() -> threads_rs::Result<()> {
    let access_token =
        std::env::var("THREADS_ACCESS_TOKEN").expect("THREADS_ACCESS_TOKEN must be set");

    let config = Config::new(
        "your-client-id",
        "your-client-secret",
        "https://example.com/callback",
    );

    let client = Client::with_token(config, &access_token).await?;

    let me = client.get_me().await?;
    println!("User ID: {}", me.id);
    println!("Username: @{}", me.username);
    if let Some(name) = &me.name {
        println!("Name: {name}");
    }
    println!("Verified: {}", me.is_verified);

    Ok(())
}
