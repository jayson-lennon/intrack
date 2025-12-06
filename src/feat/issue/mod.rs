use std::{collections::HashMap, sync::LazyLock};

use error_stack::{Report, ResultExt};
use jiff::Timestamp;
use regex::Regex;
use serde::{Deserialize, Serialize};

mod comment;
mod priority;
mod status;
mod template;

pub use comment::Comment;
pub use priority::{Priority, PriorityParseError};
pub use status::{Status, StatusParseError};
pub use template::{IssueItemTemplate, IssueTemplateError};

pub type IssueId = u64;

static RE_ISSUE_EXTRACT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)^---\n+(?P<yaml>.*)\n+---(?P<comment>.*)$"#).unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: IssueId,
    pub title: String,
    pub created: Timestamp,
    pub status: Status,
    pub priority: Priority,
    pub created_by: String,
    pub custom: HashMap<String, String>,
}

impl Issue {
    /// Generates pretty-printed JSON template for new issue.
    pub fn new(new_id: IssueId) -> Result<Option<(Issue, Comment)>, Report<IssueTemplateError>> {
        let template = r#"---
title: ENTER ISSUE TITLE HERE
created_by: YOUR.EMAIL@EXAMPLE.COM

# Trivial | Low | Medium | High | Critical | Blocker
priority: Low

custom:
  # assigned_to: user

---
<no comment provided>
"#;

        let mut edit = dialoguer::Editor::new();
        edit.require_save(true);
        edit.extension("yaml");

        let Some(issue) = edit
            .edit(template)
            .change_context(IssueTemplateError)
            .attach("failed to edit new issue template")?
        else {
            return Ok(None);
        };

        let (yaml, comment) = Self::extract_issue_parts(&issue)?;

        let issue = {
            let issue: IssueItemTemplate = serde_yaml::from_str(yaml)
                .change_context(IssueTemplateError)
                .attach("failed to deserialize new issue")?;
            Issue {
                id: new_id,
                title: issue.title,
                created: Timestamp::now(),
                status: Status::Open,
                priority: issue.priority,
                created_by: issue.created_by,
                custom: issue.custom,
            }
        };

        let comment = Comment {
            parent_issue: new_id,
            content: comment.to_string(),
            created: issue.created,
            created_by: issue.created_by.clone(),
        };

        Ok(Some((issue, comment)))
    }

    fn extract_issue_parts(issue: &str) -> Result<(&str, &str), Report<IssueTemplateError>> {
        let caps = &(*RE_ISSUE_EXTRACT)
            .captures(issue.trim())
            .ok_or(IssueTemplateError)
            .attach("No match found")?;
        let yaml = caps
            .name("yaml")
            .ok_or(IssueTemplateError)
            .attach("No yaml group")?
            .as_str()
            .trim();
        let comment = caps
            .name("comment")
            .ok_or(IssueTemplateError)
            .attach("Missing comment group")?
            .as_str()
            .trim();
        Ok((yaml, comment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_issue_parts_valid_single_line() {
        let input = r#"---
title: Fix bug
---
This is the comment."#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(yaml, "title: Fix bug");
        assert_eq!(comment, "This is the comment.");
    }

    #[test]
    fn extract_issue_parts_valid_empty_yaml() {
        let input = r#"---

---
This is the comment."#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(yaml, "");
        assert_eq!(comment, "This is the comment.");
    }

    #[test]
    fn extract_issue_parts_valid_empty_comment() {
        let input = r#"---
title: Test
---
"#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(yaml, "title: Test");
        assert_eq!(comment, "");
    }

    #[test]
    fn extract_issue_parts_valid_multiline_yaml() {
        let input = r#"---
title: Multi line
description: Some desc

priority: High
---
Comment."#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(
            yaml,
            "title: Multi line\ndescription: Some desc\n\npriority: High"
        );
        assert_eq!(comment, "Comment.");
    }

    #[test]
    fn extract_issue_parts_valid_multiline_comment() {
        let input = r#"---
title: Test
---
Line 1 of comment.
Line 2."#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(yaml, "title: Test");
        assert_eq!(comment, "Line 1 of comment.\nLine 2.");
    }

    #[test]
    fn extract_issue_parts_invalid_no_match() {
        let inputs = [
            "",
            "no---",
            "---\ntitle\n-- -\ncomment", // wrong delimiter
        ];
        for input in inputs {
            assert!(
                Issue::extract_issue_parts(input).is_err(),
                "Expected error for input: {:?}",
                input
            );
        }
    }

    #[test]
    fn extract_issue_parts_invalid_missing_second_delim() {
        let input = r#"---
title: Test"#;
        assert!(Issue::extract_issue_parts(input).is_err());
    }

    #[test]
    fn extract_issue_parts_invalid_no_newline_after_first_delim() {
        let input = r#"---title: Test\n---\ncomment"#;
        assert!(Issue::extract_issue_parts(input).is_err());
    }

    #[test]
    fn extract_issue_parts_allows_comment_to_start_on_same_line() {
        let input = r#"---
title: Test
---comment"#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        assert_eq!(yaml, "title: Test");
        assert_eq!(comment, "comment");
    }

    #[test]
    fn extract_issue_parts_multiple_delims_takes_last() {
        let input = r#"---
old yaml
---
inter
---
final yaml? No, comment"#;
        let (yaml, comment) = Issue::extract_issue_parts(input).unwrap();
        // Takes last \n---\n , yaml up to "inter"
        assert_eq!(yaml, "old yaml\n---\ninter");
        assert_eq!(comment, "final yaml? No, comment");
    }
}
