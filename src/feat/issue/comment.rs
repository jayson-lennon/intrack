use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::feat::issue::IssueId;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Comment {
    pub parent_issue: IssueId,
    pub content: String,
    pub created: Timestamp,
    pub created_by: String,
}
