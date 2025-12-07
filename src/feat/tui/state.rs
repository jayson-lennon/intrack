use crate::common::{Focus, Page};

#[derive(Debug, Default)]
pub struct TuiState {
    pub page: Page,
    pub focus: Focus,
}
