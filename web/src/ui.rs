use super::JavascriptPlayer;
use rfd::{AsyncFileDialog, FileHandle};
use ruffle_core::backend::ui::{
    DialogResultFuture, Error, FileDialogResult, FileFilter, FileSelection, FileSelectionGroup,
    MouseCursor, UiBackend,
};
use ruffle_web_common::JsResult;
use std::path::Path;
use web_sys::HtmlCanvasElement;

use chrono::{DateTime, Utc};

#[cfg(target_arch = "wasm32")]
use chrono::NaiveDateTime;

#[derive(Debug)]
struct FullScreenError {
    jsval: String,
}

impl std::fmt::Display for FullScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.jsval)
    }
}

impl std::error::Error for FullScreenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct WebFileSelection {
    handle: FileHandle,
    contents: Vec<u8>,
}

impl WebFileSelection {
    pub async fn new(handle: FileHandle) -> Self {
        let contents = handle.read().await;

        Self { handle, contents }
    }
}

fn get_extension_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| ".".to_owned() + x)
}

impl FileSelection for WebFileSelection {
    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn creation_time(&self) -> Option<DateTime<Utc>> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn creation_time(&self) -> Option<DateTime<Utc>> {
        // Creation time is not available in JS
        None
    }

    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn modification_time(&self) -> Option<DateTime<Utc>> {
        Some(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(self.handle.inner().last_modified() as i64, 0),
            Utc,
        ))
    }

    fn file_name(&self) -> Option<String> {
        Some(self.handle.file_name())
    }

    // For some reason the test suite compiles this code.
    #[cfg(not(target_arch = "wasm32"))]
    fn size(&self) -> Option<u64> {
        unreachable!();
    }

    #[cfg(target_arch = "wasm32")]
    fn size(&self) -> Option<u64> {
        Some(self.handle.inner().size() as u64)
    }

    fn file_type(&self) -> Option<String> {
        get_extension_from_filename(&self.handle.file_name())
    }

    fn creator(&self) -> Option<String> {
        None
    }

    fn contents(&self) -> &[u8] {
        &self.contents
    }

    fn write(&self, _data: &[u8]) {
        //NOOP
    }

    fn refresh(&mut self) {}
}

/// An implementation of `UiBackend` utilizing `web_sys` bindings to input APIs.
pub struct WebUiBackend {
    js_player: JavascriptPlayer,
    canvas: HtmlCanvasElement,
    cursor_visible: bool,
    cursor: MouseCursor,
    /// Is a dialog currently open
    dialog_open: bool,
}

impl WebUiBackend {
    pub fn new(js_player: JavascriptPlayer, canvas: &HtmlCanvasElement) -> Self {
        Self {
            js_player,
            canvas: canvas.clone(),
            cursor_visible: true,
            cursor: MouseCursor::Arrow,
            dialog_open: false,
        }
    }

    fn update_mouse_cursor(&self) {
        let cursor = if self.cursor_visible {
            match self.cursor {
                MouseCursor::Arrow => "auto",
                MouseCursor::Hand => "pointer",
                MouseCursor::IBeam => "text",
                MouseCursor::Grab => "grab",
            }
        } else {
            "none"
        };
        self.canvas
            .style()
            .set_property("cursor", cursor)
            .warn_on_error();
    }
}

impl UiBackend for WebUiBackend {
    fn mouse_visible(&self) -> bool {
        self.cursor_visible
    }

    fn set_mouse_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        self.update_mouse_cursor();
    }

    fn set_mouse_cursor(&mut self, cursor: MouseCursor) {
        self.cursor = cursor;
        self.update_mouse_cursor();
    }

    fn set_clipboard_content(&mut self, _content: String) {
        log::warn!("set clipboard not implemented");
    }

    fn set_fullscreen(&mut self, is_full: bool) -> Result<(), Error> {
        match self.js_player.set_fullscreen(is_full) {
            Ok(_) => Ok(()),
            Err(jsval) => Err(Box::new(FullScreenError {
                jsval: jsval
                    .as_string()
                    .unwrap_or_else(|| "Failed to change full screen state".to_string()),
            })),
        }
    }

    fn display_unsupported_message(&self) {
        self.js_player.display_unsupported_message()
    }

    fn display_root_movie_download_failed_message(&self) {
        self.js_player.display_root_movie_download_failed_message()
    }

    fn message(&self, message: &str) {
        self.js_player.display_message(message);
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
                    let mut out = Vec::with_capacity(files.len());
                    for f in files {
                        let x: Box<dyn FileSelection> = Box::new(WebFileSelection::new(f).await);
                        out.push(x);
                    }

                    FileDialogResult::Selection(FileSelectionGroup::new(out))
                } else {
                    FileDialogResult::Canceled
                }
            } else {
                let file = dialog.pick_file().await;

                if let Some(file) = file {
                    FileDialogResult::Selection(FileSelectionGroup::new(vec![Box::new(
                        WebFileSelection::new(file).await,
                    )]))
                } else {
                    FileDialogResult::Canceled
                }
            };

            Ok(result)
        }))
    }

    fn close_file_dialog(&mut self) {
        self.dialog_open = false;
    }

    fn display_file_save_dialog(
        &mut self,
        _file_name: String,
        _domain: String,
    ) -> Option<DialogResultFuture> {
        None
    }
}
