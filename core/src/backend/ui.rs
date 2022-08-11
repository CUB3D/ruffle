use crate::events::{KeyCode, PlayerEvent};
pub use crate::loader::Error as LoaderError;
use chrono::{DateTime, Utc};
use downcast_rs::Downcast;
use std::collections::HashSet;
use std::future::Future;
use std::num::{NonZeroU8, NonZeroUsize};
use std::pin::Pin;

/// Type alias for pinned, boxed, and owned futures that output a falliable
/// result of type `Result<T, E>`.
pub type OwnedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>;

/// A filter specifying a category that can be selected from a file chooser dialog
pub struct FileFilter {
    /// The description of the catagory
    pub description: String,
    /// A semicolon ';' delimited list of acceptable windows file extensions that can be selected
    /// in this category, with a */wildcard before each extension
    pub extensions: String,
    /// A semicolon ';' delimited list of acceptable MacOs file extensions that can be selected in
    /// this category, with a */wildcard before each extension
    /// Note that a list of file filters will either all have Some(_) mac_type or all will have None
    pub mac_type: Option<String>,
}

pub enum FileDialogResult {
    Selection(Box<dyn FileDialogSelection>),
    Canceled,
}

/// A result of a file selection
pub trait FileDialogSelection: Downcast {
    /// Refresh any internal metadata, any future calls to other functions (such as [FileDialogResult::size]) will reflect
    /// the state at the time of the last refresh
    fn refresh(&mut self);
    /// Get the number of files in this selection
    fn file_count(&self) -> NonZeroUsize;
    fn file(&self, id: usize) -> Option<&dyn FileSelection>;
    fn file_mut(&mut self, id: usize) -> Option<&mut dyn FileSelection>;

    /// Get the first file in the selection, as it always exists
    fn first_file(&self) -> &dyn FileSelection {
        self.file(0).expect("File selection must have at least one file")
    }

    /// Get a mutable reference to the first file in the selection, as it always exists
    fn first_file_mut(&mut self) -> &mut dyn FileSelection {
        self.file_mut(0).expect("File selection must have at least one file")
    }
}
impl_downcast!(FileDialogSelection);

pub trait FileSelection: Downcast {
    fn creation_time(&self) -> Option<DateTime<Utc>>;
    fn modification_time(&self) -> Option<DateTime<Utc>>;
    fn file_name(&self) -> Option<String>;
    fn size(&self) -> Option<u64>;
    fn file_type(&self) -> Option<String>;
    fn creator(&self) -> Option<String>;
    fn contents(&self) -> &[u8];
    /// Write the given data to the chosen file
    /// This will not necessarily by reflected in future calls to other functions (such as [FileDialogResult::size]),
    /// until [FileDialogResult::refresh] is called
    fn write(&self, data: &[u8]);
    /// Refresh any internal metadata, any future calls to other functions (such as [FileDialogResult::size]) will reflect
    /// the state at the time of the last refresh
    fn refresh(&mut self);
}
impl_downcast!(FileSelection);


/// Future representing a file selection in process
pub type DialogResultFuture = OwnedFuture<FileDialogResult, LoaderError>;

pub type Error = Box<dyn std::error::Error>;

pub trait UiBackend {
    fn mouse_visible(&self) -> bool;

    fn set_mouse_visible(&mut self, visible: bool);

    /// Changes the mouse cursor image.
    fn set_mouse_cursor(&mut self, cursor: MouseCursor);

    /// Sets the clipboard to the given content.
    fn set_clipboard_content(&mut self, content: String);

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error>;

    /// Displays a warning about unsupported content in Ruffle.
    /// The user can still click an "OK" or "run anyway" message to dismiss the warning.
    fn display_unsupported_message(&self);

    /// Displays a message about an error during root movie download.
    /// In particular, on web this can be a CORS error, which we can sidestep
    /// by providing a direct .swf link instead.
    fn display_root_movie_download_failed_message(&self);

    // Unused, but kept in case we need it later.
    fn message(&self, message: &str);

    /// Displays a file selection dialog, returning None if the dialog cannot be displayed
    /// (e.g because it is already open)
    /// * `filters` represents a list of filters to the possible file types that can be selected
    /// * `multiple_files` is true if the selection should allow selecting multiple files
    ///
    /// # Returns
    /// * `None` If the dialog cannot be displayed
    /// * If the dialog is displayed, then the returned [`Vec`], should contain at least one item
    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>, multiple_files: bool) -> Option<DialogResultFuture>;

    /// Display a dialog allowing a user to select a destination to save a file to
    ///
    /// * `file_name` is a suggestion for the file name to save the file as
    /// * `title` is a title that should be displayed in the dialog
    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        title: String,
    ) -> Option<DialogResultFuture>;

    /// Mark that any previously open dialog has been closed
    fn close_file_dialog(&mut self);
}

/// A mouse cursor icon displayed by the Flash Player.
/// Communicated from the core to the UI backend via `UiBackend::set_mouse_cursor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseCursor {
    /// The default arrow icon.
    /// Equivalent to AS3 `MouseCursor.ARROW`.
    Arrow,

    /// The hand icon incdicating a button or link.
    /// Equivalent to AS3 `MouseCursor.BUTTON`.
    Hand,

    /// The text I-beam.
    /// Equivalent to AS3 `MouseCursor.IBEAM`.
    IBeam,

    /// The grabby-dragging hand icon.
    /// Equivalent to AS3 `MouseCursor.HAND`.
    Grab,
}

pub struct InputManager {
    keys_down: HashSet<KeyCode>,
    last_key: KeyCode,
    last_char: Option<char>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            last_key: KeyCode::Unknown,
            last_char: None,
        }
    }

    fn add_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::Unknown {
            self.keys_down.insert(key_code);
        }
    }

    fn remove_key(&mut self, key_code: KeyCode) {
        self.last_key = key_code;
        if key_code != KeyCode::Unknown {
            self.keys_down.remove(&key_code);
        }
    }

    pub fn handle_event(&mut self, event: &PlayerEvent) {
        match *event {
            PlayerEvent::KeyDown { key_code, key_char } => {
                self.last_char = key_char;
                self.add_key(key_code);
            }
            PlayerEvent::KeyUp { key_code, key_char } => {
                self.last_char = key_char;
                self.remove_key(key_code);
            }
            PlayerEvent::MouseDown { button, .. } => self.add_key(button.into()),
            PlayerEvent::MouseUp { button, .. } => self.remove_key(button.into()),
            _ => {}
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn last_key_code(&self) -> KeyCode {
        self.last_key
    }

    pub fn last_key_char(&self) -> Option<char> {
        self.last_char
    }

    pub fn is_mouse_down(&self) -> bool {
        self.is_key_down(KeyCode::MouseLeft)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// UiBackend that does nothing.
pub struct NullUiBackend {}

impl NullUiBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiBackend for NullUiBackend {
    fn mouse_visible(&self) -> bool {
        true
    }

    fn set_mouse_visible(&mut self, _visible: bool) {}

    fn set_mouse_cursor(&mut self, _cursor: MouseCursor) {}

    fn set_clipboard_content(&mut self, _content: String) {}

    fn set_fullscreen(&mut self, _is_full: bool) -> Result<(), Error> {
        Ok(())
    }

    fn display_unsupported_message(&self) {}

    fn display_root_movie_download_failed_message(&self) {}

    fn message(&self, _message: &str) {}

    fn display_file_open_dialog(
        &mut self,
        _filters: Vec<FileFilter>,
        _multiple_files: bool,
    ) -> Option<DialogResultFuture> {
        Some(Box::pin(async move {
            Ok(FileDialogResult::Canceled)
        }))
    }

    fn close_file_dialog(&mut self) {}

    fn display_file_save_dialog(
        &mut self,
        _file_name: String,
        _domain: String,
    ) -> Option<DialogResultFuture> {
        None
    }
}

impl Default for NullUiBackend {
    fn default() -> Self {
        NullUiBackend::new()
    }
}
