use chrono::{DateTime, Utc};
use clipboard::{ClipboardContext, ClipboardProvider};
use rfd::{AsyncFileDialog, FileHandle, MessageButtons, MessageDialog, MessageLevel};
use ruffle_core::backend::ui::{DialogResultFuture, FileDialogResult, FileFilter, FullscreenError, LoaderError, MouseCursor, UiBackend};
use isahc::AsyncReadResponseExt;
use ruffle_core::backend::ui::{
     DownloadDialogResult, DownloadDialogResultFuture
};
use std::fs;
use std::rc::Rc;
use winit::window::Fullscreen;
use winit::window::Window;

pub struct DesktopFileDialogResult {
    handle: Option<FileHandle>,
    md: Option<fs::Metadata>,
}

impl DesktopFileDialogResult {
    /// Create a new [`DesktopFileDialogResult`] from a given file handle
    pub fn new(handle: Option<FileHandle>) -> Self {
        let md = handle.as_ref().and_then(|x| fs::metadata(x.path()).ok());
        Self { handle, md }
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

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), FullscreenError> {
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

    fn display_file_dialog(&mut self, filters: Vec<FileFilter>) -> Option<DialogResultFuture> {
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

    fn display_file_download_dialog(
        &mut self,
        url: String,
        file_name: String,
        domain: String,
    ) -> Option<DownloadDialogResultFuture> {
        // Prevent opening multiple dialogs at the same time
        if self.dialog_open {
            return None;
        }
        self.dialog_open = true;

        // Create the dialog future
        Some(Box::pin(async move {
            // Select the location to save the file to
            let dialog = AsyncFileDialog::new()
                .set_title(&format!("Select location for download from {}", domain))
                .set_file_name(&file_name);

            let file_selection = match dialog.save_file().await {
                Some(x) => x,
                None => return Ok(None),
            };

            let path = file_selection.path().to_owned();
            let initial_file_status = Box::new(DesktopFileDialogResult::new(Some(file_selection)));

            let mut http_res = match isahc::get_async(url).await {
                Ok(x) => x,
                Err(_) => return Ok(None),
            };

            let bytes = match http_res.bytes().await {
                Ok(x) => x,
                Err(_) => return Ok(None),
            };

            match fs::write(&path, &bytes) {
                Ok(_) => {}
                Err(_) => return Ok(None),
            }

            // Get file details after download for callbacks
            let post_file_status =
                Box::new(DesktopFileDialogResult::new(Some(FileHandle::wrap(path))));

            return Ok(Some(DownloadDialogResult {
                initial_dialog_result: initial_file_status,
                dialog_result: post_file_status,
                download_size: bytes.len(),
            }));
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }
}
