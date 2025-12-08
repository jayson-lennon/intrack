use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::feat::issue::Priority;

/// Template used to create an issue from a text file.
///
/// These are the minimum fields required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueItemTemplate {
    pub title: String,
    pub priority: Priority,
    pub created_by: String,
    pub custom: HashMap<String, String>,
}
