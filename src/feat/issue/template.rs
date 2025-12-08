use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wherror::Error;

use crate::feat::issue::Priority;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueItemTemplate {
    pub title: String,
    pub priority: Priority,
    pub created_by: String,
    pub custom: HashMap<String, String>,
}
