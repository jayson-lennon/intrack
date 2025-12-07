use crate::App;
use derive_more::Debug;

pub type ExternalEditorCallback = Box<dyn FnOnce(&mut App, Option<String>)>;

#[derive(Debug)]
pub struct ExternalEditorEntry {
    pub data: String,
    pub file_extension: String,
    #[debug(skip)]
    pub callback: ExternalEditorCallback,
}

#[derive(Debug, Default)]
pub struct ExternalEditor {
    entry: Option<ExternalEditorEntry>,
}

impl ExternalEditor {
    /// Launches an external editor pre-populated with the given data.
    ///
    /// The callback function allows you to edit the program state based on the user input. If
    /// `None` is returned, then this means the user did not save their work.
    ///
    /// Set the `file_extension` to help editors with syntax highlighting.
    ///
    /// # Errors
    ///
    /// Although this method only returns an `Option`, the engine will exit the program if an error
    /// occurs while trying to open `$EDITOR`, or if an error occurs managing the temp files
    /// required to gather the input.
    pub fn edit<D, E, F>(
        &mut self,
        data: D,
        file_extension: E,
        cb: F,
    ) -> Option<ExternalEditorEntry>
    where
        D: Into<String>,
        E: Into<String>,
        F: FnOnce(&mut App, Option<String>) + 'static,
    {
        let entry = ExternalEditorEntry {
            data: data.into(),
            file_extension: file_extension.into(),
            callback: Box::new(cb),
        };
        let old_entry = self.entry.take();
        self.entry = Some(entry);
        old_entry
    }

    /// Takes out an editor entry.
    pub fn take(&mut self) -> Option<ExternalEditorEntry> {
        self.entry.take()
    }
}
