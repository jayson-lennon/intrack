use serde::{Deserialize, Serialize};

use crate::feat::issue::{Comment, Issue, IssueId, Priority, Status};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueEvent {
    IssueCreated(Issue),
    CommentAdded(Comment),
    StatusChanged {
        issue_id: IssueId,
        status: Status,
    },
    PriorityChanged {
        issue_id: IssueId,
        priority: Priority,
    },
}

impl From<Issue> for IssueEvent {
    fn from(issue: Issue) -> Self {
        IssueEvent::IssueCreated(issue)
    }
}

impl From<Comment> for IssueEvent {
    fn from(comment: Comment) -> Self {
        IssueEvent::CommentAdded(comment)
    }
}
