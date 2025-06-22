//! Home Assistant API Schemas
//! The schemas module defines the data structures used to interact with the Home Assistant API.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StateCreateOrUpdate {
    pub state: String,
    pub attributes: Option<HashMap<String, String>>,
}
