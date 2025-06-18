//! Home Assistant Integration Module
//! The integration is done via HTTP JSON API.
pub mod client;
pub use client::Client;

mod http_client;
mod schemas;
