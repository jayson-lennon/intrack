/// Types that can be attached to error reports.
pub mod report {
    /// Show a suggestion.
    pub struct Suggestion(pub &'static str);

    /// Indicate that some item is missing.
    pub struct Missing(pub String);
}
