use std::str::FromStr;

use error_stack::{Report, ResultExt};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use strum::Display;
use wherror::Error;

#[derive(Debug, Error)]
#[error(debug)]
pub struct StatusParseError;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Display, Serialize)]
pub enum Status {
    /// Issue is open.
    #[default]
    Open,
    /// Issue is closed.
    Closed,
}

impl FromStr for Status {
    type Err = Report<StatusParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" | "active" | "pending" => Ok(Status::Open),
            "closed" | "done" | "finished" => Ok(Status::Closed),
            other => {
                Err(StatusParseError).attach_with(|| format!("cannot parse '{other}' into Status"))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl Visitor<'_> for StatusVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str(r#""open", "active", "pending", "closed", "done", or "finished""#)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(StatusVisitor)
    }
}
