use chrono::{DateTime, Utc};
use clipboard::{ClipboardContext, ClipboardProvider};
use rfd::{AsyncFileDialog, FileHandle, MessageButtons, MessageDialog, MessageLevel};
use ruffle_core::backend::ui::{
    DialogResultFuture, Error, FileDialogResult, FileSelectionGroup, FileFilter, FileSelection,
    LoaderError, MouseCursor, UiBackend,
};
use std::fs;
use std::num::{NonZeroU8, NonZeroUsize};
use std::rc::Rc;
use winit::window::Fullscreen;
use winit::window::Window;

pub struct DesktopFileSelection {
    files: Vec<DesktopFile>,
}

impl DesktopFileSelection {
    pub fn new(files: Vec<FileHandle>) -> Self {
        DesktopFileSelection {
            files: files.into_iter().map(|f| DesktopFile::new(f)).collect(),
        }
    }
}

impl FileSelectionGroup for DesktopFileSelection {
    fn refresh(&mut self) {
        self.files.iter_mut().for_each(|f| f.refresh());
    }

    fn file_count(&self) -> NonZeroUsize {
        self.files
            .len()
            .try_into()
            .expect("Files must have at least one entry")
    }

    fn file(&self, id: usize) -> Option<&dyn FileSelection> {
        let x: &dyn FileSelection = self.files.get(id)?;
        Some(x)
    }

    fn file_mut(&mut self, id: usize) -> Option<&mut dyn FileSelection> {
        let x: &mut dyn FileSelection = self.files.get_mut(id)?;
        Some(x)
    }
}

pub struct DesktopFile {
    handle: FileHandle,
    md: Option<fs::Metadata>,
    contents: Vec<u8>,
}

impl DesktopFile {
    /// Create a new [`DesktopFile`] from a given file handle
    pub fn new(handle: FileHandle) -> Self {
        let md = fs::metadata(handle.path()).ok();

        let contents = fs::read(handle.path()).unwrap_or_default();

        Self {
            handle,
            md,
            contents,
        }
    }
}

impl FileSelection for DesktopFile {
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
        Some(self.handle.file_name())
    }

    fn size(&self) -> Option<u64> {
        self.md.as_ref().map(|md| md.len())
    }

    fn file_type(&self) -> Option<String> {
        self.handle
            .path()
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| ".".to_owned() + x)
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write(&self, data: &[u8]) {
        let _ = fs::write(self.handle.path(), data);
    }

    fn refresh(&mut self) {
        self.contents = fs::read(self.handle.path()).unwrap_or_default();
        self.md = fs::metadata(self.handle.path()).ok()
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

    fn display_file_open_dialog(
        &mut self,
        filters: Vec<FileFilter>,
        multiple_files: bool,
    ) -> Option<DialogResultFuture> {
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

            let result = if multiple_files {
                let files = dialog.pick_files().await;

                if let Some(files) = files {
                    FileDialogResult::Selection(Box::new(DesktopFileSelection::new(files)))
                } else {
                    FileDialogResult::Canceled
                }
            } else {
                let file = dialog.pick_file().await;

                if let Some(file) = file {
                    FileDialogResult::Selection(Box::new(DesktopFileSelection::new(vec![file])))
                } else {
                    FileDialogResult::Canceled
                }
            };

            Ok(result)
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

            let result = if let Some(file) = dialog.save_file().await {
                FileDialogResult::Selection(Box::new(DesktopFileSelection::new(vec![file])))
            } else {
                FileDialogResult::Canceled
            };

            Ok(result)
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }
}
