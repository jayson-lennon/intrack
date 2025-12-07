use std::{collections::HashMap, fs::OpenOptions, io::Write, path::Path};

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use wherror::Error;

use crate::feat::issue::{Comment, Issue, IssueId};

mod event;

pub use event::IssueEvent;

#[derive(Debug, Error)]
#[error(debug)]
pub struct IssuesEventError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Issues {
    issues: HashMap<IssueId, Issue>,
    comments: HashMap<IssueId, Vec<Comment>>,
}

impl Issues {
    pub fn iter_issues(&self) -> impl Iterator<Item = &Issue> {
        self.issues.values()
    }

    pub fn iter_comments(&self) -> impl Iterator<Item = (&IssueId, &Vec<Comment>)> {
        self.comments.iter()
    }

    /// Applies a single event to update the projected state.
    pub fn apply_event(&mut self, event: IssueEvent) {
        match event {
            IssueEvent::IssueCreated(item) => {
                self.issues.insert(item.id, item);
            }
            IssueEvent::CommentAdded(comment) => {
                self.comments
                    .entry(comment.parent_issue)
                    .or_default()
                    .push(comment);
            }
            IssueEvent::StatusChanged { issue_id, status } => {
                self.issues
                    .entry(issue_id)
                    .and_modify(|issue| issue.status = status);
            }
            IssueEvent::PriorityChanged { issue_id, priority } => {
                self.issues
                    .entry(issue_id)
                    .and_modify(|issue| issue.priority = priority);
            }
        }
    }

    /// Returns the next available sequential issue ID (max(existing IDs) + 1 or 1 if empty).
    pub fn next_issue_id(&self) -> IssueId {
        self.issues.keys().map(|&id| id + 1).max().unwrap_or(1)
    }

    /// Reconstructs Issues state from an iterator of events.
    pub fn from_events(events: impl IntoIterator<Item = IssueEvent>) -> Self {
        let mut issues = Self::default();
        for event in events {
            issues.apply_event(event);
        }
        issues
    }

    /// Loads Issues state from a JSONL file where each line is an `IssueEvent`.
    pub fn from_jsonl_file<P>(path: P) -> Result<Self, Report<IssuesEventError>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .change_context(IssuesEventError)
            .attach_with(|| format!("failed to read file {:?}", path.display()))?;

        let mut events = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let event = serde_json::from_str::<IssueEvent>(line)
                .change_context(IssuesEventError)
                .attach_with(|| format!("failed to deserialize event at line {}", idx + 1))
                .attach_with(|| format!("content: {}", line))?;
            events.push(event);
        }

        Ok(Self::from_events(events))
    }

    /// Appends a new event to the event log file and applies it to the projected state.
    pub fn append_to_log<P>(
        &mut self,
        path: P,
        event: IssueEvent,
    ) -> Result<(), Report<IssuesEventError>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .change_context(IssuesEventError)
            .attach_with(|| format!("failed to open event log file {:?}", path.display()))?;

        let event_json = serde_json::to_string(&event)
            .change_context(IssuesEventError)
            .attach("failed to serialize event")?;

        file.write_all(event_json.as_bytes())
            .change_context(IssuesEventError)
            .attach("failed to write event JSONL to file")?;
        file.write_all(b"\n")
            .change_context(IssuesEventError)
            .attach("failed to write newline to file")?;

        self.apply_event(event.clone());

        Ok(())
    }
}
