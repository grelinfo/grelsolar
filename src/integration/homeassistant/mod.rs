//! Home Assistant Integration Module
//! The integration is done via HTTP JSON API.
mod client;
mod error;
mod http_client;
mod schemas;

pub use client::Client;
pub use error::{Error, Result};
