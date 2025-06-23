//! Solar-Log Integration Module
//! The integration is done via HTTP JSON API.
mod client;
mod error;
mod http_client;

pub use client::{Client, InverterStatus};
pub use error::{Error, Result};
