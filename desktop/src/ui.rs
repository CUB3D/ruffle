use chrono::{DateTime, Utc};
use clipboard::{ClipboardContext, ClipboardProvider};
use rfd::{AsyncFileDialog, FileHandle, MessageButtons, MessageDialog, MessageLevel};
use ruffle_core::backend::ui::{
    DialogResultFuture, Error, FileDialogResult, FileFilter, LoaderError, MouseCursor, UiBackend,
};
use std::fs;
use std::rc::Rc;
use winit::window::Fullscreen;
use winit::window::Window;

pub struct DesktopFileDialogResult {
    handle: Option<FileHandle>,
    md: Option<fs::Metadata>,
    contents: Vec<u8>,
}

impl DesktopFileDialogResult {
    /// Create a new [`DesktopFileDialogResult`] from a given file handle
    pub fn new(handle: Option<FileHandle>) -> Self {
        let md = handle.as_ref().and_then(|x| fs::metadata(x.path()).ok());

        let contents = handle
            .as_ref()
            .and_then(|handle| fs::read(handle.path()).ok())
            .unwrap_or_default();

        Self {
            handle,
            md,
            contents,
        }
    }
}

impl FileDialogResult for DesktopFileDialogResult {
    fn is_cancelled(&self) -> bool {
        self.handle.is_none()
    }

    fn creation_time(&self) -> Option<DateTime<Utc>> {
        if let Some(md) = &self.md {
            md.created().ok().map(DateTime::<Utc>::from)
        } else {
            None
        }
    }

    fn modification_time(&self) -> Option<DateTime<Utc>> {
        if let Some(md) = &self.md {
            md.modified().ok().map(DateTime::<Utc>::from)
        } else {
            None
        }
    }

    fn file_name(&self) -> Option<String> {
        self.handle.as_ref().map(|handle| handle.file_name())
    }

    fn size(&self) -> Option<u64> {
        self.md.as_ref().map(|md| md.len())
    }

    fn file_type(&self) -> Option<String> {
        if let Some(handle) = &self.handle {
            handle
                .path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| ".".to_owned() + x)
        } else {
            None
        }
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write(&self, data: &[u8]) {
        if let Some(handle) = &self.handle {
            let _ = fs::write(handle.path(), data);
        }
    }

    fn refresh(&mut self) {
        let md = self
            .handle
            .as_ref()
            .and_then(|x| fs::metadata(x.path()).ok());

        let contents = if let Some(handle) = &self.handle {
            fs::read(handle.path()).unwrap_or_default()
        } else {
            Vec::new()
        };

        self.md = md;
        self.contents = contents;
    }
}

pub struct DesktopUiBackend {
    window: Rc<Window>,
    cursor_visible: bool,
    clipboard: ClipboardContext,
    /// Is a dialog currently open
    dialog_open: bool,
}

impl DesktopUiBackend {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            window,
            cursor_visible: true,
            clipboard: ClipboardProvider::new().unwrap(),
            dialog_open: false,
        }
    }
}

// TODO: Move link to https://ruffle.rs/faq or similar
const UNSUPPORTED_CONTENT_MESSAGE: &str = "\
The Ruffle emulator does not yet support ActionScript 3, required by this content.
If you choose to run it anyway, interactivity will be missing or limited.

See the following link for more info:
https://github.com/ruffle-rs/ruffle/wiki/Frequently-Asked-Questions-For-Users";

const DOWNLOAD_FAILED_MESSAGE: &str = "Ruffle failed to open or download this file.";

impl UiBackend for DesktopUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
        self.cursor_visible = visible;
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        use winit::window::CursorIcon;
        let icon = match cursor {
            MouseCursor::Arrow => CursorIcon::Arrow,
            MouseCursor::Hand => CursorIcon::Hand,
            MouseCursor::IBeam => CursorIcon::Text,
            MouseCursor::Grab => CursorIcon::Grab,
        };
        self.window.set_cursor_icon(icon);
    }

    fn set_clipboard_content(&mut self, content: String) {
        self.clipboard.set_contents(content).unwrap();
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error> {
        self.window.set_fullscreen(if is_full {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        Ok(())
    }

    fn display_unsupported_message(&self) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Ruffle - Unsupported content")
            .set_description(UNSUPPORTED_CONTENT_MESSAGE)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn display_root_movie_download_failed_message(&self) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Ruffle - Load failed")
            .set_description(DOWNLOAD_FAILED_MESSAGE)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn message(&self, message: &str) {
        let dialog = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("Ruffle")
            .set_description(message)
            .set_buttons(MessageButtons::Ok);
        dialog.show();
    }

    fn display_file_open_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            let mut dialog = AsyncFileDialog::new();

            for filter in filters {
                if std::env::consts::OS == "macos" && filter.mac_type.is_some() {
                    let mac_type = filter.mac_type.unwrap();
                    let extensions: Vec<&str> = mac_type.split(';').collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                } else {
                    let extensions: Vec<&str> = filter
                        .extensions
                        .split(';')
                        .map(|x| x.trim_start_matches("*."))
                        .collect();
                    dialog = dialog.add_filter(&filter.description, &extensions);
                }
            }

            let result: Result<Box<dyn FileDialogResult>, LoaderError> = Ok(Box::new(
                DesktopFileDialogResult::new(dialog.pick_file().await),
            ));
            result
        }))
    }

    fn display_file_save_dialog(
        &mut self,
        file_name: String,
        title: String,
    ) -> Option<DialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            // Select the location to save the file to
            let dialog = AsyncFileDialog::new()
                .set_title(&title)
                .set_file_name(&file_name);

            let result: Result<Box<dyn FileDialogResult>, LoaderError> = Ok(Box::new(
                DesktopFileDialogResult::new(dialog.save_file().await),
            ));
            result
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }
}
