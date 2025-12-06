use std::str::FromStr;

use error_stack::{Report, ResultExt};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use strum::Display;
use wherror::Error;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Display, Serialize)]
pub enum Priority {
    #[default]
    /// Typos, documentation tweaks (trivial).
    Trivial,
    /// Minor bugs, UI polish, or nice-to-haves (low).
    Low,
    /// Standard bugs or enhancements with moderate impact (medium).
    Medium,
    /// Major feature gaps or bugs affecting core functionality (high).
    High,
    /// Severe bugs impacting many users; high-severity security flaws (critical).
    Critical,
    /// Blocks development/release; critical system outage or security issue (blocker).
    Blocker,
}

#[derive(Debug, Error)]
#[error(debug)]
pub struct PriorityParseError;

impl FromStr for Priority {
    type Err = Report<PriorityParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trivial" | "t" | "typo" => Ok(Priority::Trivial),
            "low" | "l" => Ok(Priority::Low),
            "medium" | "m" => Ok(Priority::Medium),
            "high" | "h" => Ok(Priority::High),
            "critical" | "c" => Ok(Priority::Critical),
            "blocker" | "b" => Ok(Priority::Blocker),
            other => Err(PriorityParseError)
                .attach_with(|| format!("cannot parse '{other}' into Priority; expected: trivial/low/medium/high/critical/blocker"))
        }
    }
}

impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PriorityVisitor;

        impl Visitor<'_> for PriorityVisitor {
            type Value = Priority;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#""trivial"(t/typo), "low"(l), "medium"(m), "high"(h), "critical"(c), "blocker"(b)""#)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(PriorityVisitor)
    }
}
