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
pub use template::IssueItemTemplate;
use wherror::Error;

/// A type alias for issue identifiers.
pub type IssueId = u64;

/// Regex pattern for extracting YAML frontmatter and comment content from issue templates.
///
/// The pattern matches the format:
/// ```plain
/// ---
/// <yaml content>
/// ---
/// <comment content>
/// ```
static RE_ISSUE_EXTRACT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?s)^---\n+(?P<yaml>.*)\n+---(?P<comment>.*)$"#).unwrap());

#[derive(Debug, Error)]
#[error(debug)]
pub struct IssueParseError;

/// Represents a project issue with metadata and custom fields.
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
    /// Returns the template that can be used to interactively create an issue.
    pub fn new_template() -> &'static str {
        r#"---
title: ENTER ISSUE TITLE HERE
created_by: YOUR.EMAIL@EXAMPLE.COM

# Trivial | Low | Medium | High | Critical | Blocker
priority: Low

custom:
  # assigned_to: user

---
<no comment provided>
"#
    }
    /// Creates a new issue by opening an editor with a YAML template.
    ///
    /// This method presents the user with an editor containing a template for the new issue.
    /// The user can edit the YAML frontmatter and add a comment. If the user saves and closes
    /// the editor, a new issue and comment are created and returned. If the user cancels without
    /// saving, `None` is returned.
    ///
    /// # Errors
    ///
    /// This function returns `Err(Report<IssueTemplateError>)` if:
    /// - The external editor fails to launch or process the template.
    /// - The edited content cannot be parsed to extract YAML frontmatter and comment.
    /// - The YAML frontmatter fails to deserialize into an `IssueItemTemplate`.
    pub fn from_str<S>(
        new_id: IssueId,
        issue: S,
    ) -> Result<Option<(Issue, Comment)>, Report<IssueParseError>>
    where
        S: AsRef<str>,
    {
        let issue = issue.as_ref();
        let (yaml, comment) = Self::extract_issue_parts(&issue)?;

        let issue = {
            let issue: IssueItemTemplate = serde_yaml::from_str(yaml)
                .change_context(IssueParseError)
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

    /// Extracts YAML frontmatter and comment content from an issue template string.
    ///
    /// Parses the template format to separate the YAML frontmatter (between the first `---` delimiters)
    /// from the comment content (after the second `---` delimiter).
    ///
    /// Returns the YAML content and comment content as separate string slices.
    ///
    /// # Errors
    ///
    /// Returns `Err(Report<IssueParseError>)` if:
    /// - The input string does not match the expected regex pattern.
    /// - The YAML frontmatter capture group is missing.
    /// - The comment capture group is missing.
    fn extract_issue_parts(issue: &str) -> Result<(&str, &str), Report<IssueParseError>> {
        let caps = &(*RE_ISSUE_EXTRACT)
            .captures(issue.trim())
            .ok_or(IssueParseError)
            .attach("No match found")?;
        let yaml = caps
            .name("yaml")
            .ok_or(IssueParseError)
            .attach("No yaml group")?
            .as_str()
            .trim();
        let comment = caps
            .name("comment")
            .ok_or(IssueParseError)
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
                "Expected error for input: {input:?}"
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
